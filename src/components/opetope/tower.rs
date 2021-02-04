use super::*;



pub struct Tower<Data> {
    cells: VersionedVec<Data>,
}



// IMPL: Initialization
//
impl<Data> Tower<Data> {
    pub fn init(root: Data) -> (ViewIndex, Self) {
        let index = unsafe { Index::from_raw_parts(0, 0) };

        (
            ViewIndex { index, depth: 0 },
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
    pub fn into_next(self) -> Diagram<Data> {
        Tail::Tower(self).into()
    }
}

// IMPL: Accessing
//
impl<Data> Tower<Data> {
    pub fn contents_of(&self, cell: index::local::Cell) -> Option<Vec<index::local::Cell>> {
        let up_cell = cell.inner() - 1;

        if self.cells.contains(up_cell) {
            Some(vec![index::local::Cell(up_cell)])

        } else {
            None
        }
    }

    pub fn collective_inputs(&self, cells: &[index::local::Cell]) -> Result<Vec<index::prev::Cell>, Error> {
        let bad = cells
            .iter()
            .filter(|cell| !self.cells.contains(cell.inner()))
            .map(|cell| cell.inner())
            .next();

        if let Some(bad) = bad {
            Err(Error::NoSuchCell(bad))

        } else {
            Ok(vec![])
        }
    }
}

// IMPL: Utils
//
impl<Data> Tower<Data> {
    pub fn level(&self) -> usize {
        0
    }

    fn valid_level(cell: ViewIndex) -> Result<Index, Error> {
        if let ViewIndex { index, depth: 0 } = cell {
            Ok(index)

        } else {
            Err(Error::TooMuchDepth(cell.depth))
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

    use crate::behavior;


    impl<Data> Tower<Data>
    where Data: behavior::SimpleView + behavior::Clickable {

        pub fn view(&mut self) -> iced::Element<Message> {
            let mut tower = self.cells.iter_mut_indices();

            let (top_idx, top_data) = tower.next().unwrap();

            let mut downmost_cell: iced::Element<_>
                = Self::end(
                    top_data,
                    top_idx,
                );

            while let Some((idx, data)) = tower.next() {
                downmost_cell
                    = Self::cell(
                        downmost_cell,
                        data,
                        idx,
                    );
            }

            downmost_cell.into()
        }
    }

    impl<Data> Tower<Data>
    where Data: behavior::SimpleView + behavior::Clickable {

        fn end(data: &mut Data, index: Index) -> iced::Element<Message> {
            let contents = data.view().map(|_| Message::Idle);

            iced::Button::new(data.state(), contents)
                .style(crate::styles::container::CELL)
                .padding(crate::styles::container::PADDING)
                .on_press(Message::Select(index))
                .into()
        }

        fn cell<'c>(contents: impl Into<iced::Element<'c, Message>>, data: &'c mut Data, index: Index) -> iced::Element<'c, Message> {
            let contents = iced::Column::new()
                .push(contents)
                .push(data.view().map(|_| Message::Idle));

            iced::Button::new(data.state(), contents)
                .style(crate::styles::container::CELL)
                .padding(crate::styles::container::PADDING)
                .on_press(Message::Select(index))
                .into()
        }
    }
}
