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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Error {
    NoSuchCell(Index),
}


enum Site {
    End { corresponding_group: index::prev::Cell },
    Group { contents: Vec<index::local::Cell> },
}

struct MetaCell<Data> {
    data: Data,

    site: Site,
}

pub struct Diagram<Data> {
    cells: VersionedVec<MetaCell<Data>>,

    prev: Tail<Data>,
}

pub struct Tower<Data> {
    cells: VersionedVec<Data>,
}

enum Tail<Data> {
    Tower(Tower<Data>),
    Diagram(Box<Diagram<Data>>),
}


// IMPL: Initialization
//
impl<Data> Tower<Data> {
    pub fn init(root: Data) -> (Index, Self) {
        (
            unsafe { Index::from_raw_parts(0, 0) },
            Self {
                cells: vec![root].into(),
            },
        )
    }
}

// IMPL: Editing
//
impl<Data: Clone> Tower<Data> {
    pub fn extrude(&mut self, cell: Index, group: Data) -> Result<Index, Error> {
        self.cells
            .try_insert(cell + 1, group)
            .map_err(|_| Error::NoSuchCell(cell))
    }

    pub fn sprout(&mut self, cell: Index, group: Data) -> Result<Index, Error> {
        self.cells
            .try_insert(cell, group)
            .map_err(|_| Error::NoSuchCell(cell))
    }

    pub fn delete(&mut self, cell: Index) -> Result<Data, Error> {
        let removed
        = self.cells
            .try_remove(cell)
            .map_err(|_| Error::NoSuchCell(cell))?
            .ok_or(Error::NoSuchCell(cell))?;

        Ok(removed)
    }
}


pub mod viewing {
    use super::*;

    use crate::behavior;



    #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub enum Message {
        Idle,
        Select(Index),
    }


    impl<Data: behavior::SimpleView> behavior::View for Tower<Data> {
        type Msg = Message;

        fn view(&self) -> iced::Element<'static, Self::Msg> {
            let mut tower = self.cells.iter_indices();

            let (top_idx, top_data) = tower.next().unwrap();

            let mut downmost_cell: iced::Element<_>
                = Self::cell(
                    top_data
                        .view()
                        .map(|_| Message::Idle)
                );

            while let Some((idx, data)) = tower.next() {
                downmost_cell
                    = Self::cell(
                        iced::Column::new()
                            .push(downmost_cell)
                            .push(data.view().map(|_| Message::Idle))
                    );
            }

            downmost_cell.into()
        }
    }

    impl<Data> Tower<Data> {
        fn cell(contents: impl Into<iced::Element<'static, Message>>) -> iced::Element<'static, Message> {
            iced::Container::new(contents)
                .style(crate::styles::container::CELL)
                .padding(crate::styles::container::PADDING)
                .into()
        }
    }
}
