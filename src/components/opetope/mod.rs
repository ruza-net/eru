use crate::behavior::SimpleView;
use crate::model::versioned_vec::*;

pub mod data;
pub mod index {
    use super::*;

    pub mod local {
        use super::*;


        #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
        pub struct Cell(pub(in super::super) Index);

        impl Cell {
            pub fn inner(&self) -> Index {
                self.0
            }

            pub fn into_prev(self) -> super::prev::Cell {
                super::prev::Cell(self.0)
            }
        }
    }

    pub mod prev {
        use super::*;

        #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
        pub struct Cell(pub(in super::super) Index);

        impl Cell {
            pub fn inner(&self) -> Index {
                self.0
            }

            pub fn into_local(self) -> super::local::Cell {
                super::local::Cell(self.0)
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
    TooMuchDepth(usize),
    NoSuchCell(Index),
    CellsDoNotFormTree(Vec<Index>),

    CannotConvertAlreadyGrouped,
}

pub enum Tail<Data> {
    Tower(Tower<Data>),
    Diagram(Box<Diagram<Data>>),
}

pub enum EditResult<O, Data> {
    Ok(O),
    OkCopied { result: O, copy: Tail<Data> },
    Err(Error),
}



macro_rules! common_methods {
    (
        $( $([$mt:ident])? $name:ident ( $($arg:ident : $t:ty),* ) $(-> $ret:ty)? ),*
    ) => {
        $(
            fn $name(&$($mt)? self $(, $arg: $t)*) $(-> $ret)? {
                match self {
                    Self::Tower(d) => d.$name($($arg),*).into(),
                    Self::Diagram(d) => d.$name($($arg),*).into(),
                }
            }
        )*
    };
}

impl<Data> Tail<Data> {
    common_methods! {
        level() -> usize,

        has_groups() -> bool,
        contents_of(cell: index::local::Cell) -> Option<Vec<index::local::Cell>>,
        collective_inputs(cells: &[index::local::Cell]) -> Result<Vec<index::prev::Cell>, Error>
    }
}

impl<Data: SimpleView> Tail<data::Selectable<Data>> {
    common_methods! {
        [mut] view() -> iced::Element<viewing::Message>
    }
}

impl<Data: Clone> Tail<Data> {
    common_methods! {
        [mut] extrude(cell: viewing::ViewIndex, group: Data) -> EditResult<viewing::ViewIndex, Data>
    }
}

impl<Data> Tail<data::Selectable<Data>> {
    common_methods! {
        [mut] select(cell: viewing::ViewIndex) -> Result<(), Error>
    }
}


impl<O, Data> EditResult<O, Data> {
    pub fn unwrap(self) -> O {
        match self {
            Self::Ok(result) => result,
            Self::OkCopied { result, .. } => result,

            Self::Err(e) => panic!["called `EditResult::unwrap` on error: {:?}", e],
        }
    }

    pub fn map<D>(self, f: impl FnOnce(O) -> D) -> EditResult<D, Data> {
        match self {
            Self::Ok(result) => EditResult::Ok(f(result)),
            Self::OkCopied { result, copy } => EditResult::OkCopied { result: f(result), copy },

            Self::Err(e) => EditResult::Err(e),
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



pub mod viewing {
    use super::*;
    use std::fmt;


    #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct ViewIndex {
        pub(in super) index: Index,
        pub(in super) depth: usize,
    }

    #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub enum Message {
        Idle,
        Select(ViewIndex),
    }


    // IMPL: Accessing
    //
    impl ViewIndex {
        pub fn inner(&self) -> Index {
            self.index
        }
    }

    impl fmt::Display for ViewIndex {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write![fmt, "[{} in layer {}]", self.index, self.depth]
        }
    }
}
