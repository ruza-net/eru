use super::{ *, viewing::{ ViewIndex, Selection, Index } };





#[derive(Debug, Clone)]
pub struct Tower<Data> {
    cells: TracingVec<Data>,
}



// IMPL: Initialization
//
impl<Data> Tower<Data> {
    pub fn init(root: Data) -> (ViewIndex, Self) {
        let index = unsafe { TimelessIndex::from_raw_parts(0) };

        (
            ViewIndex::Ground(index),
            Self {
                cells: vec![root].into(),
            },
        )
    }
}

// IMPL: Editing
//
impl<Data: Clone> Tower<Data> {
    pub fn extrude(&mut self, cell: ViewIndex, group: Data) -> Result<ViewIndex, Error> {
        let cell = Self::valid_level(cell)?;

        self.cells
            .try_insert(cell + 1, group)
            .map(|index| ViewIndex { index, depth: 0 })
            .map_err(|_| Error::NoSuchCell(cell))
    }

    pub fn sprout(&mut self, cell: ViewIndex, group: Data) -> Result<ViewIndex, Error> {
        let cell = Self::valid_level(cell)?;

        self.cells
            .try_insert(cell, group)
            .map(|index| ViewIndex { index, depth: 0 })
            .map_err(|_| Error::NoSuchCell(cell))
    }

    pub fn delete(&mut self, cell: ViewIndex) -> Result<Data, Error> {
        let cell = Self::valid_level(cell)?;

        let removed
        = self.cells
            .try_remove(cell)
            .map_err(|_| Error::NoSuchCell(cell))?
            .ok_or(Error::NoSuchCell(cell))?;

        Ok(removed)
    }
}

// IMPL: Transforming
//
impl<Data> Tower<Data> {
    pub fn into_next(self) -> Result<Diagram<Data>, Error> {
        Diagram::new(Tail::Tower(self))
    }
}

// IMPL: Accessing
//
impl<Data> Tower<Data> {
    pub const fn level(&self) -> usize {
        0
    }

    pub fn has_groups(&self) -> bool {
        self.cells.len() > 1
    }

    pub fn is_end(&self, cell: &ViewIndex) -> Result<bool, Error> {
        let index = Self::valid_level(cell)?;

        let index =
        self.cells
            .into_timed(index)
            .map_err(|_| Error::NoSuchCell(cell.clone()))?;

        Ok(
            self.cells
                .try_first_index()
                .map(|first| first == index)
                .unwrap_or(false)
        )
    }

    pub fn is_bottom(&self, cells: &Selection) -> Result<bool, Error> {
        let index = Self::valid_level(cells)?;

        let index =
        self.cells
            .into_timed(index)
            .map_err(|_| Error::NoSuchCell(ViewIndex::Ground(index)))?;

        Ok(
            self.cells
                .last_index()
                .map(|last| last == index)
                .unwrap_or(false)
        )
    }

    pub fn is_middle(&self, cell: &Selection) -> Result<bool, Error> {
        // Ok(!(self.is_end(cell)? || self.is_bottom(cell)?))

        todo!()
    }

    pub fn contents_of(&self, index: &[TimelessIndex]) -> Option<Vec<ViewIndex>> {
        let index = if let &[index] = index { index } else { None? };

        let timed =
        self.cells
            .into_timed(index)
            .ok()?;

        Some(
            self.cells
                .into_timeless(timed - 1)
                .map(ViewIndex::Ground)
                .map(|index| vec![index])
                .unwrap_or(vec![])
        )
    }
}

// IMPL: Utils
//
impl<Data> Tower<Data> {
    fn extract(sel: &Selection) -> Result<TimelessIndex, Error> {
        sel .as_ground()
            .ok_or(Error::TooMuchDepth(sel.level()))
    }

    fn valid_level(cell: &dyn super::viewing::Index) -> Result<TimelessIndex, Error> {
        if let Some(index) = cell.as_ground() {
            Ok(index)

        } else {
            Err(Error::TooMuchDepth(cell.level()))
        }
    }
}

impl<Data: Clone> Tower<Data> {
    pub fn deep_copy(&self, level: usize) -> Result<Tail<Data>, Error> {
        if level == 0 {
            Ok(Tail::Tower(self.clone()))

        } else {
            Err(Error::TooMuchDepth(level))
        }
    }
}

// IMPL: Selections
//
impl<Data> Tower<data::Selectable<Data>> {
    pub fn select(&mut self, cell: ViewIndex) -> Result<(), Error> {
        if let ViewIndex { index, depth: 0 } = cell {
            self.cells
                .get_mut(index).ok_or(Error::NoSuchCell(index))?
                .select();

            Ok(())

        } else {
            Err(Error::TooMuchDepth(cell.depth))
        }
    }
}



pub mod viewing {
    use super::*;
    use super::super::viewing::Message;

    use crate::behavior::SimpleView;


    impl<Data> Tower<data::Selectable<Data>>
    where Data: SimpleView
    {
        pub fn view(&mut self) -> iced::Element<Message> {
            let mut tower = self
                .cells
                .iter_mut_timeless_indices()
                .map(|(index, data)| (ViewIndex::Ground(index), data));

            let (top_idx, top_data) = tower.next().unwrap();

            let mut downmost_cell: iced::Element<_>
                = top_data.view_cell(top_idx, None);

            while let Some((idx, data)) = tower.next() {
                downmost_cell
                    = data.view_cell(idx, Some(downmost_cell));
            }

            downmost_cell
        }
    }
}
