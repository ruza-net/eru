use crate::behavior::SimpleView;

use crate::model::tracing_tree;
use tracing_tree::TTree;


pub mod experimental;

pub mod data;

mod tower;
pub use tower::Tower;

mod line;
mod diagram;
pub use diagram::Diagram;



#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    VecIndexError(tracing_vec::IndexError),
    TreeIndexError(tracing_tree::IndexError),

    TooMuchDepth(usize),
    CannotEditInner(usize),

    NoSuchCell(ViewIndex),
    CannotSproutGroup(ViewIndex),
    CannotExtrudeNestedCells(viewing::Selection),

    CannotGroupDisconnected(Vec<ViewIndex>),

    CellsDoNotFormTree(Vec<ViewIndex>),
    CellsDoNotHaveOutput(Vec<ViewIndex>),

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Action {
    Extrude { group: ViewIndex, contents: Vec<ViewIndex> },
    Sprout { group: ViewIndex, end: ViewIndex },
    Delete { cell: ViewIndex },

    // NOTE: Split is represented by two actions.
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Interaction {
    InPrevious { action: Action, wrap: ViewIndex },

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

        is_end(cell: &ViewIndex) -> Result<bool, Error>,
        contents_of(index: &ViewIndex) -> Option<Vec<ViewIndex>>
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
        [mut] sprout(cell: &ViewIndex, end: Data, wrap: Data) -> EditResult<Interaction, Data>
    }
}

// IMPL: Selections
//
impl<Data> Tail<data::Selectable<Data>> {
    common_methods! {
        [mut] select(cell: &ViewIndex) -> Result<Option<viewing::Selection>, Error>,
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

    pub fn and_then<U>(self, op: impl FnOnce(O, Option<Tail<Data>>) -> EditResult<U, Data>) -> EditResult<U, Data> {
        match self {
            EditResult::Ok(result) => op(result, None),
            EditResult::OkCopied { result, copy } => op(result, Some(copy)),
            EditResult::Err(err) => EditResult::Err(err),
        }
    }

    pub fn or_else(self, op: impl FnOnce(Error) -> Self) -> Self {
        match self {
            EditResult::Ok(result) => EditResult::Ok(result),
            EditResult::OkCopied { result, copy } => EditResult::OkCopied { result, copy },
            EditResult::Err(err) => op(err),
        }
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


pub use viewing::{ Message, ViewIndex, Selection };
pub mod viewing {
    use super::*;
    use std::fmt;

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum ViewIndex {
        Ground(tracing_vec::TimelessIndex),
        Leveled { level: usize, path: tracing_tree::TimedIndex },
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Selection {
        Ground(tracing_vec::TimelessIndex),
        Leveled { level: usize, sel: tracing_tree::Subtree },
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Message {
        Idle,
        Select(ViewIndex),
    }

    pub(in super) trait Index {
        fn as_ground(&self) -> Option<tracing_vec::TimelessIndex>;
        fn as_subtree(&self) -> Option<tracing_tree::Subtree>;
        fn level(&self) -> usize;
    }


    // IMPL: Accessing
    //
    impl ViewIndex {
        pub fn to_path(self) -> Option<tracing_tree::TimedIndex> {
            match self {
                ViewIndex::Ground(_) => None,
                ViewIndex::Leveled { path, .. } => Some(path),
            }
        }

        pub fn path(&self) -> Option<&tracing_tree::TimedIndex> {
            match self {
                ViewIndex::Ground(_) => None,
                ViewIndex::Leveled { path, .. } => Some(path),
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

                Selection::Leveled { level, sel } =>
                    sel
                        .iter()
                        .cloned()
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
        fn as_ground(&self) -> Option<tracing_vec::TimelessIndex> {
            match self {
                Self::Ground(index) => Some(*index),
                Self::Leveled { .. } => None,
            }
        }
        fn as_subtree(&self) -> Option<tracing_tree::Subtree> {
            match self {
                ViewIndex::Ground(_) => None,
                ViewIndex::Leveled { path, .. } => Some(path.clone().into()),
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
        fn as_ground(&self) -> Option<tracing_vec::TimelessIndex> {
            match self {
                Self::Ground(index) => Some(*index),
                Self::Leveled { .. } => None,
            }
        }
        fn as_subtree(&self) -> Option<tracing_tree::Subtree> {
            match self {
                Selection::Ground(index) => None,

                Selection::Leveled { sel, .. } => Some(sel.clone()),
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
                    path,
                ],
            }
        }
    }
}
