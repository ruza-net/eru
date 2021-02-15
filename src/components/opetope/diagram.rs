use super::{ *, viewing::{ ViewIndex, Selection, Index } };

use std::collections::HashSet;



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



#[derive(Debug, Clone)]
struct Face {
    ends: Vec<ViewIndex>,
    fill: ViewIndex,
}

#[derive(Debug, Clone)]
pub struct MetaCell<Data> {
    data: Data,
    face: Face,

    content: Option<TracingVec<Self>>,
}


#[derive(Debug, Clone)]
pub struct Diagram<Data> {
    cells: TracingVec<MetaCell<Data>>,

    prev: Tail<Data>,
}



// IMPL: Initialization
//
impl<Data> Diagram<Data> {
    pub fn new(tail: Tail<Data>) -> Result<Self, Error> {

        if tail.has_groups() {
            return Err(Error::CannotConvertAlreadyGrouped);
        }

        Ok(Self {
            prev: tail,
            cells: fill![],
        })
    }
}

// IMPL: Accessing
//
impl<Data> Diagram<Data> {
    pub fn level(&self) -> usize {
        self.prev.level() + 1
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn get(&self, path: &[TimelessIndex]) -> Option<&MetaCell<Data>> {
        let mut acc =
        self.cells
            .get(*path.first()?)
            .ok()?;

        for &seg in &path[1..] {
            acc = acc.get(seg)?;
        }

        Some(acc)
    }

    pub fn get_mut(&mut self, path: &[TimelessIndex]) -> Option<&mut MetaCell<Data>> {
        let mut acc =
        self.cells
            .get_mut(*path.first()?)
            .ok()?;

        for &seg in &path[1..] {
            acc = acc.get_mut(seg)?;
        }

        Some(acc)
    }

    pub fn has_groups(&self) -> bool {
        self
            .cells
            .iter()
            .any(|cell| cell.is_group())
    }
}

// IMPL: Transforming
//
impl<Data> Diagram<Data> {
    pub fn into_next(self) -> Result<Diagram<Data>, Error> {
        if let Ok(ret) = Self::new(Tail::Diagram(Box::new(self))) {
            Ok(ret)

        } else {
            todo!()
        }
    }
}

// IMPL: Utils
//
impl<Data> Diagram<Data> {
    fn cell_with_end(&self, end: &ViewIndex) -> Option<TimedIndex> {
        self.cells
            .iter_indices()
            .filter(|(_, cell)| cell.face.ends.contains(end))
            .map(|(index, _)| index)
            .next()
    }

    fn cell_space_mut(&mut self, path: &[TimelessIndex]) -> &mut TracingVec<MetaCell<Data>> {
        if self.get(path).is_some() {
            let owner =
            self.get_mut(path)
                .unwrap();

            owner
                .content
                    .as_mut()
                    .unwrap()

        } else {
            &mut self.cells
        }
    }

    fn all_inputs(&self) -> Vec<ViewIndex> {
        let outputs: HashSet<_> =
        self.cells
            .iter()
            .map(|cell| &cell.face.fill)
            .collect();

        self.cells
            .iter()
            .map(|cell| cell.face.ends.to_vec())
            .flatten()
            .filter(|input| !outputs.contains(input))
            .collect()
    }

    fn into_index(&self, path: Vec<TimelessIndex>) -> ViewIndex {
        ViewIndex::Leveled {
            level: self.level() - 1,

            path,
        }
    }

    fn check_form_tree<'c>(&self, cells: &'c [Vec<TimelessIndex>]) -> Result<(&'c [TimelessIndex], Vec<TimelessIndex>), Error> {
        let ret = self.check_cells_connected(cells)?;

        let inputs: HashSet<_> =
        cells
            .iter()
            .map(|path| &self.get(path).unwrap().face.ends)
            .flatten()
            .collect();

        let dangling =
        cells
            .iter()
            .map(|path| &self.get(path).unwrap().face.fill)
            .filter(|line| !inputs.contains(line))
            .count();

        if dangling == 1 {
            Ok(ret)

        } else {
            Err(Error::CellsDoNotFormTree(
                cells
                    .iter()
                    .cloned()
                    .map(|path| self.into_index(path))
                    .collect()
            ))
        }
    }

    fn check_cells_connected<'c>(&self, cells: &'c [Vec<TimelessIndex>]) -> Result<(&'c [TimelessIndex], Vec<TimelessIndex>), Error> {
        let heads: Vec<_> =
        cells
            .iter()
            .map(|path| path.last().unwrap())
            .copied()
            .collect();

        let mut tails: Vec<_> =
        cells
            .iter()
            .map(|path| &path[.. path.len() - 1])
            .collect();

        let tail =
        tails
            .pop()
            .unwrap();

        if tails
            .into_iter()
            .all(|tl| tail == tl)
        {
            Ok((tail, heads))

        } else {
            Err(Error::CannotGroupDisconnected(
                cells
                    .iter()
                    .cloned()
                    .map(|path| self.into_index(path))
                    .collect()
            ))
        }
    }

    pub fn contents_of(&self, cell: &[TimelessIndex]) -> Option<Vec<ViewIndex>> {
        self.get(cell)
            .map(|cell| cell.content.as_ref())
            .flatten()
            .map(|tr_vec|
                tr_vec
                    .timeless_indices()
                    .map(|index| self.into_index({
                        let mut path = cell.to_vec();
                        path.push(index);

                        path
                    }))
                    .collect()
            )
    }

    pub fn is_end(&self, cell: &ViewIndex) -> Result<bool, Error> {
        let path = self.valid_level(cell)?;

        Ok(
            self.get(&path)
                .ok_or(Error::NoSuchCell(cell.clone()))?
                .content
                .is_none()
        )
    }

    fn valid_level(&self, index: &ViewIndex) -> Result<Vec<TimelessIndex>, Error> {
        if self.level() == index.level() {
            Ok(index.path())

        } else {
            Err(Error::TooMuchDepth(index.level()))
        }
    }
}

impl<Data: Clone> Diagram<Data> {
    pub fn deep_copy(&self, level: usize) -> Result<Tail<Data>, Error> {
        if self.level() == level {
            Ok(Tail::Diagram(Box::new(self.clone())))

        } else {
            self.prev.deep_copy(level - 1)
        }
    }
}

// IMPL: Selections
//
impl<Data> Diagram<data::Selectable<Data>> {
    pub fn select(&mut self, cell: &ViewIndex) -> Result<Option<Selection>, Error> {
        if self.level() == cell.level() {
            self.prev.unselect_all(0);

            if let Some(selection) = self.selected_cells() {
                if selection.common_path() != cell.tail() {
                    self.unselect_all(self.level());
                }
            }

            let cell =
            self
                .get_mut(&cell.path())
                .ok_or(Error::NoSuchCell(cell.clone()))?;

            cell.data
                .select();

            Ok(self.selected_cells())

        } else if self.level() > cell.level() {
            self.unselect_all(self.level());

            self.prev
                .select(cell)

        } else {
            Err(Error::TooMuchDepth(cell.level()))
        }
    }

    pub fn unselect_all(&mut self, max_depth: usize) {
        if max_depth < self.level() {
            self.prev.unselect_all(max_depth);
        }

        self.cells
            .iter_mut()
            .for_each(|cell| cell.unselect_all())
    }

    fn selected_cells(&self) -> Option<Selection> {
        let all_selected =
        self.cells
            .iter_timeless_indices()
            .map(|(index, cell)| {
                cell.selected_cells()
                    .into_iter()
                    .map(move |mut path| {
                        path.insert(0, index.clone());
                        path
                    })
            })
            .fold(vec![], |mut acc, paths|
            {
                acc.extend(paths);

                acc
            });

        let first = all_selected.first()?;

        let path = first[.. first.len() - 1].to_vec();
        let level = self.level() - 1;

        assert![all_selected.iter().all(|p| path == p[.. p.len() - 1])];

        let cells =
        all_selected
            .into_iter()
            .map(|mut path| path.pop().unwrap())
            .collect();

        Some(Selection::Leveled {
            level,
            path,
            cells,
        })
    }
}


// IMPL: Accessing
//
impl<Data> MetaCell<Data> {
    fn is_group(&self) -> bool {
        self.content.is_some()
    }

    fn get(&self, seg: TimelessIndex) -> Option<&Self> {
        self.content
            .as_ref()?
            .get(seg)
            .ok()
    }

    fn get_mut(&mut self, seg: TimelessIndex) -> Option<&mut Self> {
        self.content
            .as_mut()?
            .get_mut(seg)
            .ok()
    }
}

// IMPL: Selections
//
impl<Data> MetaCell<data::Selectable<Data>> {
    fn unselect_all(&mut self) {
        self.data.unselect();

        if let Some(content) = &mut self.content {
            content
                .iter_mut()
                .for_each(|cell| cell.unselect_all())
        }
    }

    fn selected(&self) -> bool {
        self.data.selected()
    }

    fn selected_cells(&self) -> Vec<Vec<TimelessIndex>> {
        if self.selected() {
            vec![vec![]]

        } else if let Some(cells) = &self.content {
            cells
                .iter_timeless_indices()
                .map(|(index, cell)| (index, cell.selected_cells()))
                .fold(vec![], |mut acc, (index, selected)|
                {
                    acc.extend(
                        selected
                            .into_iter()
                            .map(|mut sl| { &mut sl.insert(0, index); sl })
                    );

                    acc
                })

        } else {
            vec![]
        }
    }
}

impl Face {
    fn collect<'c>(cells: impl Iterator<Item = &'c Self>) -> Self {
        let mut ends = vec![];
        let mut fills = vec![];

        for cell in cells {
            ends.extend(cell.ends.iter());

            fills.push(&cell.fill);
        }

        fills =
        fills
            .into_iter()
            .filter(|fill| {
                if let Some(pos) = ends.iter().position(|end| end == fill) {
                    ends.remove(pos);

                    false

                } else {
                    true
                }
            })
            .collect();

        if let [fill] = fills[..] {
            let ends =
            ends.into_iter()
                .cloned()
                .collect();

            let fill = fill.clone();

            Face { ends, fill }

        } else {
            unreachable![]
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
