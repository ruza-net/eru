use super::{
    *,
    viewing::{ ViewIndex, Selection, Index }
};
use serde::{ Serialize, Deserialize };


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
                    .map(|timed| $self.cells.into_timeless(timed).unwrap())
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



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tower<Data> {
    cells: TracingVec<Data>,
}



// IMPL: Initialization
//
impl<Data> Tower<Data> {
    pub fn init(root: Data) -> (ViewIndex, Self) {
        let cells = TracingVec::from(vec![root]);
        let index = cells.timeless_indices().next().unwrap();

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
        self.split(extract[sel: Selection], group: Data, _wrap_top: Data, _wrap_bot: Data)
            => Split { group, contents: self.contents_of(&group.path()).unwrap() }
            => try_insert_after(group)

        where if !is_middle => CannotSplitBoundaryCells
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

    pub(in super) fn is_end(&self, cell: &dyn super::viewing::Index) -> Result<bool, Error> {
        let index = Self::valid_level(cell)?;

        let index =
        self.cells
            .into_timed(index)
            .map_err(|_| Error::NoSuchCell(ViewIndex::Ground(index)))?;

        Ok(
            self.cells
                .try_first_index()
                .map(|first| self.cells.indices_eq(first, index).unwrap())
                .unwrap_or(false)
        )
    }

    pub(in super) fn is_bottom(&self, cells: &Selection) -> Result<bool, Error> {
        let index = Self::valid_level(cells)?;

        let index =
        self.cells
            .into_timed(index)
            .map_err(|_| Error::NoSuchCell(ViewIndex::Ground(index)))?;

        Ok(
            self.cells
                .try_last_index()
                .map(|last| self.cells.indices_eq(last, index).unwrap())
                .unwrap_or(false)
        )
    }

    pub(in super) fn is_middle(&self, cell: &Selection) -> Result<bool, Error> {
        let index = Self::valid_level(cell)?;

        Ok(!self.is_bottom(&Selection::Ground(index))?)
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

    pub fn rename(&mut self, cell: &ViewIndex, new_data: Data) -> Result<(), Error> {
        let index = Self::valid_level(cell)?;

        self.cells
            .get_mut(index)
            .map(|cell| *cell = new_data)
            .map_err(|_| Error::NoSuchCell(cell.clone()))
    }
}

impl<Data: Clone> Tower<Data> {
    pub fn cell(&self, cell: &ViewIndex) -> Result<Cell<Data>, Error> {
        let index = Self::valid_level(cell)?;

        self.cells
            .get(index)
            .map(|data| Cell::Ground(data.clone()))
            .map_err(|e| Error::IndexError(e))
    }
}

// IMPL: Utils
//
impl<Data> Tower<Data> {
    fn extract(sel: &dyn super::viewing::Index) -> Result<TimelessIndex, Error> {
        sel
            .as_ground()
            .ok_or(Error::TooMuchDepth(sel.level()))
    }

    fn valid_level(cell: &dyn super::viewing::Index) -> Result<TimelessIndex, Error> {
        if let Some(index) = cell.as_ground() {
            Ok(index)

        } else {
            Err(Error::TooMuchDepth(cell.level()))
        }
    }

    pub fn is_before(&self, before: &ViewIndex, after: &ViewIndex) -> bool {
        let before = before.as_ground().unwrap();
        let after = after.as_ground().unwrap();

        self.cells.is_before(before, after).unwrap()
    }

    pub fn is_at_bottom(&self, cells: &Selection) -> Result<bool, Error> {
        let index = Self::valid_level(cells)?;

        Ok(self.cells.into_timeless(self.cells.last_index()).unwrap() == index)
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

    pub fn selected_cells(&self) -> Option<Selection> {
        self.cells
            .iter_timeless_indices()
            .filter(|(_, cell)| cell.selected())
            .map(|(index, _)| Selection::Ground(index))
            .next()
    }
}



pub mod viewing {
    use super::*;
    use super::super::viewing::Message;

    use crate::model::Render;
    use crate::behavior::SimpleView;


    use crate::styles::container::PADDING;

    impl<Data> Tower<data::Selectable<Data>>
    where Data: SimpleView
    {
        pub fn view(&mut self, render: Render) -> iced::Element<Message> {
            let mut tower = self
                .cells
                .iter_mut_timeless_indices()
                .map(|(index, data)| (ViewIndex::Ground(index), data));

            let (top_idx, top_data) = tower.next().unwrap();

            let ((_, mut width), mut downmost_cell) = top_data.view_cell(top_idx, 0, None, render);


            while let Some((idx, data)) = tower.next() {
                let ((_, new_width), new_downmost_cell) =
                data.view_cell(
                    idx,
                    width,
                    Some(
                        iced::Container::new(downmost_cell)
                        .padding(PADDING)
                        .into()
                    ),
                    render
                );

                width = new_width;
                downmost_cell = new_downmost_cell;
            }

            downmost_cell =
            iced::Container::new(downmost_cell)
                .width(iced::Length::Shrink)
                .into();

            downmost_cell
        }
    }
}
