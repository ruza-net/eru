use super::*;



enum Site {
    End { corresponding_group: index::prev::Cell },
    Group { contents: Vec<index::local::Cell> },
}

pub struct MetaCell<Data> {
    data: Data,

    site: Site,
}


pub struct Diagram<Data> {
    cells: VersionedVec<MetaCell<Data>>,

    prev: Tail<Data>,
}



// IMPL: Accessing
//
impl<Data> Diagram<Data> {
    pub fn get(&self, cell: index::local::Cell) -> Option<&MetaCell<Data>> {
        self.cells.get(cell.inner())
    }

    pub fn contents_of(&self, cell: index::local::Cell) -> Option<Vec<index::local::Cell>> {
        self.get(cell)?
            .site
            .contents()
            .map(|c| c.to_vec())
    }

    pub fn collective_inputs(&self, cells: &[index::local::Cell]) -> Result<Vec<index::prev::Cell>, Error> {
        self.check_cells_form_tree(cells)?;

        Ok(cells
            .iter()
            .map(|cell| self.inputs_of(*cell).unwrap())
            .flatten()
            .collect())
    }

    pub fn inputs_of(&self, cell: index::local::Cell) -> Option<Vec<index::prev::Cell>> {
        let cell = cell.inner();

        let cell = self.cells.get(cell)?;

        match &cell.site {
            Site::End { corresponding_group } => Some(
                self.prev
                    .contents_of(corresponding_group.into_local())?
                    .into_iter()
                    .map(|idx| idx.into_prev())
                    .collect()
                ),

            Site::Group { contents } => Some(
                self.prev
                    .collective_inputs(contents).ok()?
                    .into_iter()
                    .collect()
            ),
        }
    }
}

// IMPL: Private Utils
//
impl<Data> Diagram<Data> {
    fn check_cells_form_tree(&self, cells: &[index::local::Cell]) -> Result<(), Error> {
        todo![]
    }
}


impl Site {
    fn contents(&self) -> Option<&[index::local::Cell]> {
        match self {
            Self::End { .. } => None,
            Self::Group { contents } => Some(contents),
        }
    }
}



pub mod viewing {
    use super::*;
    use crate::components::opetope::{
        viewing::Message,
        line,
    };

    use crate::behavior;


    impl<Data> Diagram<Data>
    where Data: behavior::SimpleView + behavior::Clickable {

        pub fn view(&mut self) -> iced::Element<Message> {
            todo![]
        }
    }

    impl<Data> Diagram<Data>
    where Data: behavior::SimpleView + behavior::Clickable {

        fn end(&mut self, index: Index) -> iced::Element<Message> {
            let inputs = self.inputs_of(index::local::Cell(index)).unwrap();

            todo![]
        }
    }
}
