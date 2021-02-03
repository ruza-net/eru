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
    NoSuchCell(Index),
    CellsDoNotFormTree(Vec<Index>),
}

enum Tail<Data> {
    Tower(Tower<Data>),
    Diagram(Box<Diagram<Data>>),
}


macro_rules! common_methods {
    ( $( $name:ident ( $($arg:ident : $t:ty),* ) $(-> $ret:ty)? ),* ) => {
        impl<Data> Tail<Data> {
            $(
                fn $name(&self $(, $arg: $t)*) $(-> $ret)? {
                    match self {
                        Self::Tower(d) => d.$name($($arg)*),
                        Self::Diagram(d) => d.$name($($arg)*),
                    }
                }
            )*
        }
    };
}

common_methods! {
    contents_of(cell: index::local::Cell) -> Option<Vec<index::local::Cell>>,
    collective_inputs(cells: &[index::local::Cell]) -> Result<Vec<index::prev::Cell>, Error>
}



pub mod viewing {
    use super::*;


    #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub enum Message {
        Idle,
        Select(Index),
    }
}
