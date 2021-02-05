use super::*;


macro_rules! dispatch_level {
    (
        $(
            $v:vis fn $name:ident (&mut $self:ident, { $idx:ident, $dpth:ident } : ViewIndex $(, $arg:ident : $t:ty)* ) throws [$err:expr] $(-> $ret:ty)? $body:block
        )*
    ) => {
        $(
            $v fn $name(
                &mut $self,
                ViewIndex { index: $idx, depth: $dpth }: ViewIndex
                $(, $arg: $t)*
            ) $(-> $ret)? {

                if $self.level > $dpth {
                    $self.prev.$name(ViewIndex { index: $idx, depth: $dpth } $(, $arg)*)

                } else if $self.level == $dpth
                    $body

                else {
                    $err
                }
            }
        )*
    };
}



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

    level: usize,
    prev: Tail<Data>,
}



// IMPL: Accessing
//
impl<Data> Diagram<Data> {
    pub fn get(&self, cell: index::local::Cell) -> Option<&MetaCell<Data>> {
        self.cells.get(cell.inner())
    }

    pub fn get_mut(&mut self, cell: index::local::Cell) -> Option<&mut MetaCell<Data>> {
        self.cells.get_mut(cell.inner())
    }

    pub fn has_groups(&self) -> bool {
        todo!()
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

// IMPL: Transforming
//
impl<Data> Diagram<Data> {
    pub fn into_next(self) -> Result<Diagram<Data>, Error> {
        Ok(Tail::Diagram(Box::new(self.is_rooted()?)).into())
    }
}

// IMPL: Editing
//
impl<Data: Clone> Diagram<Data> {
    dispatch_level! {
        pub fn extrude(&mut self, { index, depth }: ViewIndex, group: Data) throws[EditResult::Err(Error::TooMuchDepth(depth))] -> EditResult<ViewIndex, Data> {
            todo![]
        }
    }
}

// IMPL: Utils
//
impl<Data> Diagram<Data> {
    fn check_cells_form_tree(&self, cells: &[index::local::Cell]) -> Result<(), Error> {
        todo![]
    }

    fn is_rooted(self) -> Result<Self, Error> {
        todo![]
    }

    pub fn level(&self) -> usize {
        self.level // NOTE: Can be replaced by `self.prev.level() + 1` to enable stabilization.
    }
}

// IMPL: Selections
//
impl<Data> Diagram<data::Selectable<Data>> {
    dispatch_level! {
        pub fn select(&mut self, { index, depth }: ViewIndex) throws[Err(Error::TooMuchDepth(depth))] -> Result<(), Error> {
            let cell = self.get_mut(index::local::Cell(index)).ok_or(Error::NoSuchCell(index))?;

            cell.data.select();

            Ok(())

        }
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


impl<Data> From<Tail<Data>> for Diagram<Data> {
    fn from(prev: Tail<Data>) -> Self {
        let level = prev.level() + 1;

        Self {
            cells: fill![],

            level,
            prev,
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
