use super::{ *, viewing::{ ViewIndex, Selection, Index } };


macro_rules! interaction {
    (
        $self:ident . $name:ident ( $verify:ident [$ref_name:ident : $ref_ty:ty] $(, $arg:ident : $arg_ty:ty)* )
        => $action:ident { $field:ident $(: $override:expr)? $(, $rest:ident $(: $r_override:expr)?)* }
        => $method:ident ( $($call_arg:ident),* )
        $(where if !$invariant:ident => $err:ident)?
    ) => {
        pub fn $name(&mut $self, $ref_name: & $ref_ty $(, $arg: $arg_ty)*) -> EditResult<Interaction, Data> {
            if let Ok(index) = Self::$verify($ref_name) {
                $(
                    match $self.$invariant($ref_name) {
                        Ok($invariant) =>
                            if !$invariant {
                                return EditResult::Err(Error::$err($ref_name.clone()));
                            },

                        Err(e) => return EditResult::Err(e),
                    }
                )?

                match
                $self
                    .cells
                    .$method(index $(, $call_arg)*)
                    .map(|timed| self.cells.into_timeless(timed).unwrap())
                    .map(ViewIndex::Ground)
                    .map_err(|e| Error::IndexError(e))
                {
                    Ok($field) =>
                        EditResult::Ok(Interaction::Here {
                            action: Action::$action { $($rest $(: $r_override)?, )* $field $(: $override)? },
                        }),

                    Err(e) =>
                        EditResult::Err(e),
                }

            } else {
                EditResult::Err(Error::TooMuchDepth($ref_name.level()))
            }
        }
    };
}



#[derive(Debug, Clone)]
pub struct Tower<Data> {
    cells: TracingVec<Data>,
}



// IMPL: Initialization
//
impl<Data> Tower<Data> {
    pub fn init(root: Data) -> (ViewIndex, Self) {
        let cells = TracingVec::from(vec![root]);
        let index = cells.first_index();

        (
            ViewIndex::Ground(index),
            Self {
                cells,
            },
        )
    }
}

// IMPL: Editing
//
impl<Data> Tower<Data> {
    interaction! {
        self.extrude(extract[sel: Selection], group: Data, _wrap: Data)
            => Extrude { group, contents: self.contents_of(&group.path()).unwrap() }
            => try_insert_after(group)

        where if !is_bottom => CannotExtrudeNestedCells
    }

    interaction! {
        self.sprout(valid_level[index: ViewIndex], end: Data, _wrap: Data)
            => Sprout { end, group: index.clone() }
            => try_insert_before(end)

        where if !is_end => CannotSproutGroup
    }

    interaction! {
        self.delete(valid_level[index: ViewIndex])
            => Delete { cell }
            => try_remove()
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
    pub fn select(&mut self, cell: &ViewIndex) -> Result<Option<Selection>, Error> {
        let index = Self::valid_level(cell)?;

        let selected =
        self.cells
            .get_mut(index)
            .map_err(|_| Error::NoSuchCell(cell.clone()))?
            .selected();

        self.unselect_all(0);

        if !selected {
            self.cells
                .get_mut(index)
                .unwrap()
                .select();

            Ok(Some(Selection::Ground(index)))

        } else {
            Ok(None)
        }
    }

    pub fn unselect_all(&mut self, _max_depth: usize) {
        self.cells
            .iter_mut()
            .for_each(|cell| cell.unselect())
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
