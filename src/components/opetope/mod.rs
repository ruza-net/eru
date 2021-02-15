use crate::behavior::SimpleView;
use crate::model::tracing_vec::*;

pub mod data;
pub mod index {
    use super::*;

    pub mod local {
        use super::*;


        #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
        pub struct Cell(pub(in super::super) viewing::ViewIndex);

        impl Cell {
            pub fn into_prev(self) -> super::prev::Cell {
                super::prev::Cell(self.0)
            }
        }

        impl From<viewing::ViewIndex> for Cell {
            fn from(addr: viewing::ViewIndex) -> Self {
                Self(addr)
            }
        }
    }

    pub mod prev {
        use super::*;

        #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
        pub struct Cell(pub(in super::super) viewing::ViewIndex);

        impl Cell {
            pub fn into_local(self) -> super::local::Cell {
                super::local::Cell(self.0)
            }
        }

        impl From<viewing::ViewIndex> for Cell {
            fn from(addr: viewing::ViewIndex) -> Self {
                Self(addr)
            }
        }
    }
}

mod tower;
pub use tower::Tower;

mod line;
mod diagram;
pub use diagram::Diagram;



#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Error {
    IndexError(IndexError),

    TooMuchDepth(usize),
    CannotEditInner(usize),

    NoSuchCell(viewing::ViewIndex),
    CannotSproutGroup(viewing::ViewIndex),
    CannotExtrudeNestedCells(viewing::Selection),

    CannotGroupDisconnected(Vec<viewing::ViewIndex>),

    CellsDoNotFormTree(Vec<viewing::ViewIndex>),
    CellsDoNotHaveOutput(Vec<TimelessIndex>),

    NoLayerBeneath,
    CannotConvertAlreadyGrouped,
}

#[derive(Debug, Clone)]
pub enum Tail<Data> {
    Tower(Tower<Data>),
    Diagram(Box<Diagram<Data>>),
}

#[must_use = "this `EditResult` might be an `Err` variant which should be handled"]
#[derive(Debug, Clone)]
pub enum EditResult<O, Data> {
    Ok(O),
    OkCopied { result: O, copy: Tail<Data> },
    Err(Error),
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Action {
    Extrude { group: viewing::ViewIndex },
    Sprout { group: viewing::ViewIndex },
    Delete { cell: viewing::ViewIndex },

    // NOTE: Split is represented by two actions.
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Interaction {
    InPrevious { action: Action, wrap: viewing::ViewIndex },

    Here { action: Action },
}



macro_rules! common_methods {
    (
        $( $vs:vis $([$mt:ident])?  $name:ident ( $($arg:ident : $t:ty),* ) $(-> $ret:ty)? ),*
    ) => {
        $(
            $vs fn $name(&$($mt)? self $(, $arg: $t)*) $(-> $ret)? {
                match self {
                    Self::Tower(d) => d.$name($($arg),*).into(),
                    Self::Diagram(d) => d.$name($($arg),*).into(),
                }
            }
        )*
    };
}

// IMPL: Utils
//
impl<Data> Tail<Data> {
    common_methods! {
        level() -> usize,

        has_groups() -> bool,

        is_end(cell: &viewing::ViewIndex) -> Result<bool, Error>,
        contents_of(index: &[TimelessIndex]) -> Option<Vec<viewing::ViewIndex>>
    }
}

impl<Data: Clone> Tail<Data> {
    common_methods! {
        deep_copy(level: usize) -> Result<Tail<Data>, Error>
    }
}

// IMPL: Viewing
//
impl<Data: SimpleView> Tail<data::Selectable<Data>> {
    common_methods! {
        [mut] view() -> iced::Element<viewing::Message>
    }
}

// IMPL: Editing
//
impl<Data: Clone> Tail<Data> {
    common_methods! {
        [mut] extrude(cell: &viewing::Selection, group: Data, wrap: Data) -> EditResult<Interaction, Data>,
        [mut] sprout(cell: &viewing::ViewIndex, end: Data, wrap: Data) -> EditResult<Interaction, Data>
    }
}

// IMPL: Selections
//
impl<Data> Tail<data::Selectable<Data>> {
    common_methods! {
        [mut] select(cell: &viewing::ViewIndex) -> Result<Option<viewing::Selection>, Error>,
        [mut] unselect_all(max_depth: usize)
    }
}


impl<O, Data> EditResult<O, Data> {
    #[track_caller]
    pub fn unwrap(self) -> O {
        match self {
            Self::Ok(result) => result,
            Self::OkCopied { result, .. } => result,

            Self::Err(e) => panic!["called `EditResult::unwrap` on error: {:?}", e],
        }
    }

    pub fn ok(self) -> Result<O, Error> {
        match self {
            EditResult::Ok(result) => Ok(result),
            EditResult::OkCopied { result, .. } => Ok(result),

            EditResult::Err(e) => Err(e),
        }
    }

    pub fn map<D>(self, f: impl FnOnce(O) -> D) -> EditResult<D, Data> {
        match self {
            Self::Ok(result) => EditResult::Ok(f(result)),
            Self::OkCopied { result, copy } => EditResult::OkCopied { result: f(result), copy },

            Self::Err(e) => EditResult::Err(e),
        }
    }

    pub fn inspect_err(self, f: impl FnOnce(&Error)) -> EditResult<O, Data> {
        if let Self::Err(e) = &self {
            f(e);
        }

        self
    }
}

impl<O, Data> From<Result<O, Error>> for EditResult<O, Data> {
    fn from(res: Result<O, Error>) -> EditResult<O, Data> {
        match res {
            Ok(o) => Self::Ok(o),
            Err(e) => Self::Err(e),
        }
    }
}



pub mod viewing {
    use super::*;
    use std::fmt;

    #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub enum ViewIndex {
        Ground(TimelessIndex),
        Leveled { level: usize, path: Vec<TimelessIndex> },
    }

    #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub enum Selection {
        Ground(TimelessIndex),
        Leveled { level: usize, path: Vec<TimelessIndex>, cells: Vec<TimelessIndex> },
    }

    #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub enum Message {
        Idle,
        Select(ViewIndex),
    }

    pub(in super) trait Index {
        fn as_ground(&self) -> Option<TimelessIndex>;
        fn as_paths(&self) -> Vec<Vec<TimelessIndex>>;
        fn level(&self) -> usize;
    }


    // IMPL: Accessing
    //
    impl ViewIndex {
        pub fn path(&self) -> Vec<TimelessIndex> {
            match self {
                Self::Ground(index) => vec![*index],
                Self::Leveled { path, .. } => path.clone(),
            }
        }

        pub fn tail(&self) -> Vec<TimelessIndex> {
            match self {
                ViewIndex::Ground(_) => vec![],
                ViewIndex::Leveled { path, .. } => path[.. path.len() - 1].to_vec(),
            }
        }
    }

    // IMPL: Accessing
    //
    impl Selection {
        pub fn common_path(&self) -> Vec<TimelessIndex> {
            match self {
                Self::Ground(_) => vec![],
                Self::Leveled { path, .. } => path.clone(),
            }
        }
    }

    // IMPL: Iteration
    //
    impl Selection {
        pub fn cells(&self) -> Vec<ViewIndex> {
            match self {
                Selection::Ground(index) =>
                    vec![ViewIndex::Ground(*index)],

                Selection::Leveled { level, path, cells } =>
                    cells
                        .iter()
                        .copied()
                        .map(|cell| {
                            let mut path = path.clone();
                            path.push(cell);

                            path
                        })
                        .map(|path| ViewIndex::Leveled { level: *level, path })
                        .collect()
            }
        }
    }

    impl Index for ViewIndex {
        fn level(&self) -> usize {
            match self {
                Self::Ground(_) => 0,
                Self::Leveled { level, .. } => level + 1,
            }
        }
        fn as_ground(&self) -> Option<TimelessIndex> {
            match self {
                Self::Ground(index) => Some(*index),
                Self::Leveled { .. } => None,
            }
        }
        fn as_paths(&self) -> Vec<Vec<TimelessIndex>> {
            match self {
                ViewIndex::Ground(index) => vec![vec![*index]],
                ViewIndex::Leveled { path, .. } => vec![path.clone()],
            }
        }
    }
    impl Index for Selection {
        fn level(&self) -> usize {
            match self {
                Self::Ground(_) => 0,
                Self::Leveled { level, .. } => level + 1,
            }
        }
        fn as_ground(&self) -> Option<TimelessIndex> {
            match self {
                Self::Ground(index) => Some(*index),
                Self::Leveled { .. } => None,
            }
        }
        fn as_paths(&self) -> Vec<Vec<TimelessIndex>> {
            match self {
                Selection::Ground(index) =>
                    vec![vec![*index]],

                Selection::Leveled { path, cells, .. } =>
                    cells
                        .iter()
                        .copied()
                        .map(|cell| {
                            let mut tail = path.clone();
                            tail.push(cell);

                            tail
                        })
                        .collect()
            }
        }
    }


    impl fmt::Display for ViewIndex {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Ground(idx) => write![fmt, "↓{}", idx],

                Self::Leveled { level, path } => write![fmt,
                    "{}↑{}",
                    level + 1,
                    path.iter()
                        .map(|seg| seg.to_string())
                        .collect::<Vec<_>>()
                        .join("."),
                ],
            }
        }
    }
}
