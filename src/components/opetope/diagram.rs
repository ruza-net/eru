use itertools::Itertools;
use crate::utils::{ EncapsulateIter, ProjectIter };

use serde::{ Serialize, Deserialize };
use super::{ *, viewing::{ ViewIndex, Selection, Index } };



macro_rules! interaction {
    ( $name:ident ( &mut $self:ident, $cell:ident : &$sel_ty:ty $(, $arg:ident : $arg_ty:ty)* $(,)? ) in prev $prev_body:block in self $this_body:block ) => {
        pub fn $name(&mut $self, $cell: &$sel_ty $(, $arg: $arg_ty)* ) -> EditResult<Interaction, Data> {
            if $self.level() > $cell.level() + 1 {
                let mut copy = $self.deep_copy($cell.level() + 1).unwrap();

                let result = copy.$name($cell $(, $arg)*);

                match result {
                    EditResult::Ok(result) => EditResult::OkCopied { result, copy },

                    EditResult::OkCopied { .. } => unreachable![],

                    e => e,
                }

            } else if $self.level() < $cell.level() {
                EditResult::Err(Error::TooMuchDepth($cell.level()))

            } else if $self.level() == $cell.level()
                $this_body

            else
                $prev_body
        }
    };
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    pub ends: Vec<ViewIndex>,
    pub fill: ViewIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCell<Data> {
    pub data: Data,
    pub face: Face,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(in super) struct Cell<Data> {
    pub meta: MetaCell<Data>,

    pub content: Option<TracingVec<Self>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram<Data> {
    cells: TracingVec<Cell<Data>>,

    prev: Tail<Data>,
}



// IMPL: Initialization
//
impl<Data> Diagram<Data> {
    pub fn new(tail: Tail<Data>) -> Result<Self, Error> {
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

    pub(in super) fn get(&self, path: &[TimelessIndex]) -> Option<&Cell<Data>> {
        let mut acc =
        self.cells
            .get(*path.first()?)
            .ok()?;

        for &seg in &path[1..] {
            acc = acc.get(seg)?;
        }

        Some(acc)
    }

    pub(in super) fn get_mut(&mut self, path: &[TimelessIndex]) -> Option<&mut Cell<Data>> {
        let mut acc =
        self.cells
            .get_mut(*path.first()?)
            .ok()?;

        for &seg in &path[1..] {
            acc = acc.get_mut(seg)?;
        }

        Some(acc)
    }

    pub fn iter_groups(&self) -> IterGroups<Data> {
        IterGroups {
            level: self.level() - 1,
            cells: self
                .cells
                .iter_timeless_indices()
                .map(|(index, cell)| (vec![index], cell))
                .collect(),
        }
    }

    pub fn iter_mut_groups(&mut self) -> IterMutGroups<Data> {
        IterMutGroups {
            level: self.level(),
            cells: self
                .cells
                .iter_mut_timeless_indices()
                .map(|(index, cell)| (vec![index], cell))
                .collect(),
        }
    }
}

impl<Data: Clone> Diagram<Data> {
    pub fn cell(&self, cell: &ViewIndex) -> Result<super::Cell<Data>, Error> {
        if cell.level() == self.level() {
            let path = self.valid_level(cell).unwrap();

            self.get(&path)
                .map(|cell| super::Cell::Leveled(cell.meta.clone()))
                .ok_or(Error::NoSuchCell(cell.clone()))

        } else if cell.level() < self.level() {
            self.prev.cell(cell)

        } else {
            Err(Error::TooMuchDepth(cell.level()))
        }
    }
}

// IMPL: Transforming
//
impl<Data> Diagram<Data> {
    pub fn into_next(&mut self, wraps: Vec<MetaCell<Data>>) -> Result<(), Error> {
        take_mut::take(self, |this| {
            let mut next = Self {
                prev: Tail::Diagram(Box::new(this)),
                cells: fill![],
            };

            for wrap in wraps {
                let MetaCell { data, face: Face { ends, fill } } = wrap;

                next.wrap_extrusion(data, &ends, fill);// NOTE: If this panics, program aborts.
            }

            next
        });

        Ok(())
    }
}

// IMPL: Editing
//
impl<Data: Clone> Diagram<Data> {
    interaction!{
    extrude(&mut self, cells: &Selection, group: Data, wrap: Data)
        in prev {
            self
                .prev
                .extrude(cells, group, wrap.clone())
                .map(|inter| {
                    let (fill, ends) = extract![inter => group, contents in Interaction::Here { action: Action::Extrude { group, contents } }];

                    cells
                        .as_cells()
                        .iter()
                        .zip(ends.iter())
                        .for_each(|(cell, end)| {
                            self.replace_line(&cell.path(), &end.path()).unwrap()
                        });

                    let wraps = vec![self.wrap_extrusion(wrap, &ends, fill.clone())];

                    Interaction::InPrevious {
                        wraps,
                        action: Action::Extrude { group: fill, contents: ends },
                    }
                })
                .into()
        }

        in self {
            if cells.common_path().len() > 0 {
                return EditResult::Err(Error::CannotExtrudeNestedCells(cells.clone()));
            }

            match self.group(&cells.as_paths(), group.into()) {
                Ok((group, contents)) => {
                        EditResult::Ok(Interaction::Here {
                            action: Action::Extrude { group, contents },
                        })
                    },

                Err(e) => EditResult::Err(e),
            }
        }
    }

    interaction!{
    split(&mut self, cells: &Selection, group: Data, wrap_top: Data, wrap_bot: Data)
        in prev {
            self
                .prev
                .split(cells, group, wrap_top.clone(), wrap_bot.clone())
                .map(|inter| {
                    let (fill, ends) = extract![inter => group, contents in Interaction::Here { action: Action::Split { group, contents } }];

                    cells
                        .as_cells()
                        .iter()
                        .zip(ends.iter())
                        .for_each(|(cell, end)| {
                            self.replace_line(&cell.path(), &end.path()).unwrap()
                        });

                    let wraps = self.wrap_split(wrap_top, wrap_bot, &ends, fill.clone()).to_vec();

                    Interaction::InPrevious {
                        wraps,
                        action: Action::Extrude { group: fill, contents: ends },
                    }
                })
                .into()
        }

        in self {
            match self.group(&cells.as_paths(), group.into()) {
                Ok((group, contents)) =>
                    EditResult::Ok(Interaction::Here {
                        action: Action::Split { group, contents },
                    }),

                Err(e) =>
                    EditResult::Err(e),
            }
        }
    }

    interaction!{
    sprout(&mut self, index: &ViewIndex, end: Data, wrap: Data)
        in prev {
            self
                .prev
                .sprout(index, end, wrap.clone())
                .map(|inter| {
                    let (fill, end) = extract![inter => group, end in Interaction::Here { action: Action::Sprout { group, end } }];

                    self.replace_line(&index.path(), &fill.path()).unwrap();

                    let wraps = vec![self.wrap_sprout(wrap.into(), &fill, end.clone())];

                    Interaction::InPrevious {
                        wraps,
                        action: Action::Sprout { group: fill, end },
                    }
                })
                .into()
        }

        in self {
            match self.is_end(index) {
                Ok(is_end) =>
                    if !is_end {
                        return EditResult::Err(Error::CannotSproutGroup(index.clone()));
                    },

                Err(e) => return EditResult::Err(e),
            }

            if let Some(cell) = self.get_mut(&index.path()) {
                let face = cell.face().clone();

                let end = Cell {
                    meta:
                        MetaCell {
                            data: end.into(),

                            face,
                        },

                    content: None,
                };

                let content = TracingVec::from(vec![end]);
                let innermost_index = content.into_timeless(content.first_index()).unwrap();

                cell.content = Some(content);

                let mut path = index.path();

                path.push(innermost_index);

                let level = self.level() - 1;

                EditResult::Ok(Interaction::Here {
                    action: Action::Sprout { group: index.clone(), end: ViewIndex::Leveled { level, path } },
                })

            } else {
                EditResult::Err(Error::NoSuchCell(index.clone()))
            }
        }
    }
}

impl<Data: Clone> Diagram<Data> {
    fn group(&mut self, cells: &[Vec<TimelessIndex>], data: Data) -> Result<(ViewIndex, Vec<ViewIndex>), Error> {
        let (tail, cells) = self.check_form_tree(cells)?;

        let cell_space = self.cell_space_mut(tail);

        let face = Face::collect(
            cell_space
                .iter_timeless_indices()
                .filter(|(index, _)| cells.contains(index))
                .map(|(_, cell)| cell.face())
        );

        let mut inner = vec![];

        let index =
        cell_space
            .try_replace_at_last_with(cells, |cells| {
                let content = TracingVec::from(cells);

                inner = content.timeless_indices().collect();

                Cell { meta: MetaCell { data, face }, content: Some(content) }
            })
            .map_err(Error::IndexError)?;


        let index =
        cell_space
            .into_timeless(index)
                .unwrap();

        let mut group_path = tail.to_vec();
        group_path.push(index);


        let mut contents_indices = vec![];

        for i in inner {
            let mut member_path = group_path.clone();
            member_path.push(i);

            contents_indices.push(self.into_index(member_path));
        }

        Ok((self.into_index(group_path), contents_indices))
    }
}

impl<Data> Diagram<Data> {
    fn wrap_sprout(&mut self, data: Data, old_end_line: &ViewIndex, sprouted_line: ViewIndex) -> ViewIndex {
        let mut cells_below =
        self.cells
            .iter_indices();

        let mut cells_with_fill_before = cells_below
            .take_while_ref(|(_, cell)|
                !cell.face().ends.contains(old_end_line)
            )
            .encapsulate();

        // NOTE: `cells_below` are now the downstream cells.
        // NOTE: `cells_with_fill_before` are the upstream cells and the parallel cells.

        let cells_with_end_after = cells_with_fill_before
            .take_while_ref(|(_, cell)| !self.has_fill_before(&cell.meta, old_end_line))
            .encapsulate();

        // NOTE: `cells_with_fill_before` are now cells between the downstream cells and the
        // preceding cells.

        // NOTE: `cells_with_end_after` are the upstream cells and the parallel cells running
        // into the ends to the right of `old_end_line`.

        let mut cells_below = cells_below.proj_l();
        let mut cells_with_end_after = cells_with_end_after.proj_l();

        let cells_with_fill_before = cells_with_fill_before.proj_l();

        let wrap = Cell {
            meta:
                MetaCell {
                    data,
                    face: Face {
                        ends: vec![sprouted_line],
                        fill: old_end_line.clone(),
                    },
                },

            content: None,
        };

        let inserted =
        if let Some(preceding) = cells_with_fill_before.last() {
            self.cells.insert_after(preceding, wrap)

        } else if let Some(succeeding) = cells_with_end_after.next().or(cells_below.next()) {
            self.cells.insert_before(succeeding, wrap)

        } else {
            self.cells.push(wrap);

            self.cells.last_index()
        };

        let index =
        self.cells
            .into_timeless(inserted)
            .unwrap();

        self.into_index(vec![index])
    }

    fn wrap_extrusion(&mut self, data: Data, old_fill_lines: &[ViewIndex], created_line: ViewIndex) -> ViewIndex {
        let mut upstream_outputs = self.sort_lines(old_fill_lines);

        let lowest_fill_line = upstream_outputs.pop().unwrap();

        let mut cells_next_to =
        self.cells
            .iter_indices();

        let cells_above = cells_next_to
            .take_while_ref(|(_, cell)| cell
                .face()
                .ends
                .iter()
                .all(|end| self.prev.is_before(end, &lowest_fill_line))
            )
            .proj_l();

        let cells_after = cells_next_to
            .take_while_ref(|(_, cell)| self.prev.is_before(&lowest_fill_line, &cell.face().fill))
            .proj_l();

        let mut cells_next_to = cells_next_to.proj_l();

        let wrap = Cell {
            meta:
                MetaCell {
                    data,
                    face: Face {
                        ends: old_fill_lines.to_vec(),
                        fill: created_line,
                    },
                },

            content: None,
        };


        let index =
        if let Some(cell_after) = cells_next_to.next().or(cells_after.last()) {
            self.cells.insert_before(cell_after, wrap)

        } else if let Some(cell_before) = cells_above.last() {
            self.cells.insert_after(cell_before, wrap)

        } else {
            self.cells.push(wrap);

            self.cells.last_index()
        };

        let index =
        self.cells
            .into_timeless(index)
            .unwrap();

        self.into_index(vec![index])
    }

    fn wrap_split(&mut self, wrap_top: Data, wrap_bot: Data, upstream_lines: &[ViewIndex], created_line: ViewIndex) -> [ViewIndex; 2] {
        let (path, old_wrap, prefix, postfix) = self.cell_with_inputs_mut(upstream_lines).unwrap();

        let wrap_top = Cell {
            meta:
                MetaCell {
                    data: wrap_top,
                    face: Face {
                        ends: upstream_lines.to_vec(),
                        fill: created_line.clone(),
                    },
                },

            content: None,
        };

        let mut ends = prefix;
        ends.push(created_line);
        ends.extend(postfix);

        let wrap_bot = Cell {
            meta:
                MetaCell {
                    data: wrap_bot,
                    face: Face {
                        ends,
                        fill: old_wrap.face().fill.clone(),
                    },
                },

            content: None,
        };

        let content = TracingVec::from(vec![wrap_top, wrap_bot]);

        let top = content.into_timeless(content.first_index()).unwrap();
        let bot = content.into_timeless(content.last_index()).unwrap();

        let top = {
            let mut path = path.clone();

            path.push(top);

            path
        };
        let bot = {
            let mut path = path;

            path.push(bot);

            path
        };

        old_wrap.content = Some(content);

        [self.into_index(top), self.into_index(bot)]
    }
}

// IMPL: Cell manipulation
//
impl<Data> Diagram<Data> {
    pub fn rename(&mut self, cell: &ViewIndex, new_data: Data) -> Result<(), Error> {
        if cell.level() == self.level() {
            let path = self.valid_level(cell).unwrap();

            let mut cell = self.get_mut(&path).ok_or(Error::NoSuchCell(cell.clone()))?;
            cell.meta.data = new_data;

            Ok(())

        } else if cell.level() < self.level() {
            self.prev.rename(cell, new_data)

        } else {
            Err(Error::TooMuchDepth(cell.level()))
        }
    }
}

// IMPL: Utils
//
impl<Data> Diagram<Data> {
    fn replace_line(&mut self, line: &[TimelessIndex], new: &[TimelessIndex]) -> Result<(), Error> {
        for cell in self.cells.iter_mut() {
            cell.face_mut().fill.subst_prefix(line, new);

            cell.face_mut()
                .ends
                .iter_mut()
                .for_each(|end| end.subst_prefix(line, new));

            cell.replace_line(line, new)?;
        }

        Ok(())
    }


    fn sort_lines(&self, lines: &[ViewIndex]) -> Vec<ViewIndex> {
        let mut lines = lines.to_vec();

        lines.sort_by(|a, b| {
            if self.prev.is_before(a, b) {
                std::cmp::Ordering::Less

            } else {
                std::cmp::Ordering::Greater
            }
        });

        lines
    }

    fn has_fill_before(&self, cell: &MetaCell<Data>, end: &ViewIndex) -> bool {
        self.prev.is_before(&cell.face.fill, end)
    }

    fn cell_with_inputs_mut(&mut self, inputs: &[ViewIndex]) -> Result<(Vec<TimelessIndex>, &mut Cell<Data>, Vec<ViewIndex>, Vec<ViewIndex>), Error> {
        for (index, cell) in self.cells.iter_mut_timeless_indices() {
            if cell.is_end() {
                for (i, sub) in cell.face().ends.windows(inputs.len()).enumerate() {
                    if sub == inputs {
                        let prefix = cell.face().ends[.. i].to_vec();
                        let postfix = cell.face().ends[i + inputs.len() ..].to_vec();

                        return Ok((vec![index], cell, prefix, postfix));
                    }
                }

            } else if let Ok((mut path, cell, prefix, postfix)) = cell.cell_with_inputs_mut(inputs) {
                path.insert(0, index);

                return Ok((path, cell, prefix, postfix))
            }
        }

        Err(Error::NoCellWithInputs(inputs.to_vec()))
    }


    fn cell_space(&self, path: &[TimelessIndex]) -> &TracingVec<Cell<Data>> {
        if let Some(owner) = self.get(path) {
            owner
                .content
                    .as_ref()
                    .unwrap()

        } else {
            &self.cells
        }
    }

    fn cell_space_mut(&mut self, path: &[TimelessIndex]) -> &mut TracingVec<Cell<Data>> {
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


    fn input_count(&self) -> usize {
        let outputs =
        self.cells
            .iter()
            .map(|cell| &cell.face().fill)
            .collect_vec();

        self.cells
            .iter()
            .map(|cell| &cell.face().ends)
            .flatten()
            .filter(|line| !outputs.contains(line))
            .count()
    }


    fn into_index(&self, path: Vec<TimelessIndex>) -> ViewIndex {
        ViewIndex::Leveled {
            level: self.level() - 1,

            path,
        }
    }

    fn check_form_tree<'c>(&self, cells: &'c [Vec<TimelessIndex>]) -> Result<(&'c [TimelessIndex], Vec<TimelessIndex>), Error> {
        let ret = self.check_cells_connected(cells)?;

        let inputs: Vec<_> =
        cells
            .iter()
            .map(|path| &self.get(path).unwrap().face().ends)
            .flatten()
            .collect();

        let dangling =
        cells
            .iter()
            .map(|path| &self.get(path).unwrap().face().fill)
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
}

/// Common opetope interface
///
impl<Data> Diagram<Data> {
    pub fn is_end(&self, cell: &ViewIndex) -> Result<bool, Error> {
        let path = self.valid_level(cell)?;

        Ok(
            self.get(&path)
                .ok_or(Error::NoSuchCell(cell.clone()))?
                .content
                .is_none()
        )
    }

    pub fn is_before(&self, before: &ViewIndex, after: &ViewIndex) -> bool {
        let mut before = self.valid_level(before).unwrap();
        let mut after = self.valid_level(after).unwrap();

        let common_path = Self::remove_common_prefix(&mut before, &mut after);

        let cell_space = self.cell_space(&common_path);

        if let ([before, ..], [after, ..]) = (&before[..], &after[..]) {
            cell_space.is_before(*before, *after).unwrap()

        } else {
            true
        }
    }

    pub fn is_at_bottom(&self, cells: &Selection) -> Result<bool, Error> {
        if cells.level() > self.level() {
            Err(Error::TooMuchDepth(cells.level()))

        } else if cells.level() == self.level() {
            let bottom =
            self.cells
                .timeless_indices()
                .map(|index| self.into_index(vec![index]))
                .collect_vec();

            Ok(
                cells
                    .as_cells()
                    .iter()
                    .all(|cell| bottom.contains(cell))
            )

        } else {
            self.prev.is_at_bottom(cells)
        }
    }

    fn remove_common_prefix<X>(path_a: &mut Vec<X>, path_b: &mut Vec<X>) -> Vec<X> where X: PartialEq {
        path_a.reverse();
        path_b.reverse();

        let mut common = vec![];

        while !path_a.is_empty() && !path_b.is_empty() && path_a.last() == path_b.last() {
            let x = path_a.pop().unwrap();
            path_b.pop();

            common.push(x);
        }

        path_a.reverse();
        path_b.reverse();

        common
    }
}


/// Validation
///
impl<Data> Diagram<Data> {
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
            self.prev.deep_copy(level)
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

            cell.data_mut()
                .select();// TODO: Select all cells between `cell` and the boundary of `self.selected_cells()`.

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

    pub fn selected_cells(&self) -> Option<Selection> {
        if let Some(sel) = self.prev.selected_cells() {
            return Some(sel);
        }

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
impl<Data> Cell<Data> {
    const fn is_end(&self) -> bool {
        !self.is_group()
    }

    const fn is_group(&self) -> bool {
        self.content.is_some()
    }


    const fn face(&self) -> &Face {
        &self.meta.face
    }

    fn face_mut(&mut self) -> &mut Face {
        &mut self.meta.face
    }


    const fn data(&self) -> &Data {
        &self.meta.data
    }

    fn data_mut(&mut self) -> &mut Data {
        &mut self.meta.data
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

impl<Data> MetaCell<Data> {
    pub const fn face(&self) -> &Face {
        &self.face
    }

    pub const fn data(&self) -> &Data {
        &self.data
    }
}

// IMPL: Utils
//
impl<Data> Cell<Data> {
    fn cell_with_inputs_mut(&mut self, inputs: &[ViewIndex]) -> Result<(Vec<TimelessIndex>, &mut Cell<Data>, Vec<ViewIndex>, Vec<ViewIndex>), Error> {
        if let Some(cells) = &mut self.content {
            for (index, cell) in cells.iter_mut_timeless_indices() {
                if cell.is_end() {
                    for (i, sub) in cell.face().ends.windows(inputs.len()).enumerate() {
                        if sub == inputs {
                            let prefix = cell.face().ends[.. i].to_vec();
                            let postfix = cell.face().ends[i + inputs.len() ..].to_vec();

                            return Ok((vec![index], cell, prefix, postfix));
                        }
                    }

                } else if let Ok((mut path, cell, prefix, postfix)) = cell.cell_with_inputs_mut(inputs) {
                    path.insert(0, index);

                    return Ok((path, cell, prefix, postfix))
                }
            }
        }

        Err(Error::NoCellWithInputs(inputs.to_vec()))
    }

    fn replace_line(&mut self, line: &[TimelessIndex], new: &[TimelessIndex]) -> Result<(), Error> {
        if let Some(content) = &mut self.content {
            for cell in content.iter_mut() {
                cell.face_mut().fill.subst_prefix(line, new);

                cell.face_mut()
                    .ends
                    .iter_mut()
                    .for_each(|end| end.subst_prefix(line, new));

                cell.replace_line(line, new)?;
            }
        }

        Ok(())
    }
}

// IMPL: Selections
//
impl<Data> Cell<data::Selectable<Data>> {
    fn unselect_all(&mut self) {
        self.data_mut().unselect();

        if let Some(content) = &mut self.content {
            content
                .iter_mut()
                .for_each(|cell| cell.unselect_all())
        }
    }

    fn selected(&self) -> bool {
        self.data().selected()
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
    use std::ops;

    use crate::components::opetope::{
        Spacer,
        viewing::Message,
    };

    use crate::behavior;
    use crate::model::Render;

    use crate::styles::container::{
        PADDING,
        LINE_WIDTH,
        cell::{ SPACING },
    };



    fn pad<'e>(e: impl Into<iced::Element<'e, Message>>) -> iced::Element<'e, Message> {
        iced::Row::with_children(vec![padder(), e.into(), padder()])
        .spacing(PADDING)
        .into()
    }

    fn padder() -> iced::Element<'static, Message> {
        iced::Space::new(0.into(), 0.into()).into()
    }

    pub fn view_line(height: u16) -> iced::Element<'static, Message> {
        iced::Container::new(
            iced::Space::new(LINE_WIDTH.into(), height.into())
        )
        .style(crate::styles::container::LINE)
        .into()
    }

    #[cfg(debug_assertions)]
    fn spacer(width: u16, left: bool) -> iced::Element<'static, Message> {
        iced::Container::new(
            iced::Space::new(width.into(), 3.into())
        )
        .style(if left { crate::styles::container::DEBUG_1 } else { crate::styles::container::DEBUG_2 })
        .into()
    }

    #[cfg(not(debug_assertions))]
    fn spacer(width: u16) -> iced::Element<'static, Message> {
        iced::Space::with_width(width.into())
        .into()
    }


    const DEFAULT_WIDTH: u16 = 0;

    #[derive(Debug, Clone)]
    struct LineMargin {
        upstream: Vec<Self>,

        min_left: u16,
        min_right: u16,

        delta: u16,
    }
    impl Default for LineMargin {
        fn default() -> Self {
            Self {
                min_left: DEFAULT_WIDTH / 2,
                min_right: DEFAULT_WIDTH / 2,

                upstream: vec![],

                delta: 0,
            }
        }
    }
    impl ops::AddAssign<u16> for LineMargin {
        fn add_assign(&mut self, rhs: u16) {
            self.delta += rhs.saturating_sub(self.min_left + self.min_right + self.delta);
        }
    }
    impl LineMargin {
        fn margin(&self) -> u16 {
            self.delta / 2 +
            self.up_margins().max
            (self.min_left + self.min_right)
        }

        fn left_delta(&self) -> u16 {
            self.delta / 2
        }

        fn right_delta(&self) -> u16 {
            self.delta - self.left_delta()
        }

        fn left_margin(&self) -> u16 {
            self.left_delta() +
            self.up_margins().max
            (self.min_left)
        }

        fn right_margin(&self) -> u16 {
            self.right_delta() +
            self.up_margins().max
            (self.min_right)
        }

        fn up_margins(&self) -> u16 {
            // NOTE: Correct because `margin` is `width / 2`.
            //
            self.upstream.iter().map(LineMargin::margin).sum()
        }

        fn flatten(self) -> Vec<Self> {
            let left_delta = self.left_delta();
            let right_delta = self.right_delta();

            let Self { mut upstream, mut min_left, mut min_right, .. } = self;

            min_left += left_delta;
            min_right += right_delta;

            upstream = upstream.into_iter().map(Self::flatten).flatten().collect();

            if upstream.is_empty() {
                vec![Self { upstream, min_left, min_right, delta: 0 }]

            } else {
                let up_width = upstream.iter().map(Self::margin).sum::<u16>();

                upstream[0].min_left += min_left.saturating_sub(up_width / 2);
                upstream.last_mut().unwrap().min_right += min_right.saturating_sub(up_width - up_width / 2);

                upstream
            }
        }
    }

    #[derive(Debug, Default, Clone)]
    struct LineSystem {
        lines: Vec<LineMargin>,
    }
    macro_rules! impl_index {
        ( $($typ:ty => $($idx:ty)*),* ) => {
            $(
                $(
                    impl ops::Index<$idx> for LineSystem {
                        type Output = $typ;

                        fn index(&self, index: $idx) -> &Self::Output {
                            &self.lines[index]
                        }
                    }

                    impl ops::IndexMut<$idx> for LineSystem {
                        fn index_mut(&mut self, index: $idx) -> &mut Self::Output {
                            &mut self.lines[index]
                        }
                    }
                )*
            )*
        };
    }
    impl_index! {
        LineMargin => usize,
        [LineMargin] => ops::Range<usize> ops::RangeFrom<usize> ops::RangeFull ops::RangeInclusive<usize> ops::RangeTo<usize> ops::RangeToInclusive<usize>
    }
    impl LineSystem {
        fn new(count: usize) -> Self {
            Self {
                lines: vec![fill![]; count],
            }
        }

        fn group(&mut self, count: usize) {
            let upstream = self.lines.split_off(self.lines.len() - count);

            self.lines.push(LineMargin { upstream, ..fill![] });
        }

        fn compute_deltas(&mut self, downstream: &Self) {
            self.lines
                .iter_mut()
                .zip(
                    downstream
                        .lines
                        .iter()
                )
                .map(|(this, down)| this.delta = down.margin().saturating_sub(this.margin()))
                .collect()
        }

        fn flatten(self) -> Self {
            let lines =
            self.lines
                .into_iter()
                .map(LineMargin::flatten)
                .flatten()
                .collect();

            Self { lines }
        }

        fn spaces_between(&self) -> Vec<u16> {
            let mut spaces = vec![];

            for bounds in self.lines.windows(2) {
                let space = PADDING.max(bounds[0].up_margins() + bounds[1].up_margins());

                spaces.push(space);
            }

            spaces
        }

        fn deltas(&self) -> Vec<u16> {
            let mut spaces = vec![];

            for bounds in self.lines.windows(2) {
                let space = bounds[0].delta + bounds[1].delta;
                
                spaces.push(space);
            }

            spaces
        }

        fn spaces_around(&self, widths: &[u16]) -> Vec<u16> {
            let mut spaces = vec![];

            let count = widths.len();
            
            let mut lines =
            self.lines
                .iter()
                .zip(widths)
                .map(|(line, width)| (line.clone(), *width))
                .collect_vec();

            lines.insert(0, (fill![], DEFAULT_WIDTH));
            lines.push((fill![], DEFAULT_WIDTH));

            for (i, win) in lines.windows(2).enumerate() {
                let in_between = (1 .. count).contains(&i);
                let resolve = move |margin| {
                    margin + if in_between { PADDING } else { DEFAULT_WIDTH }
                };

                let ((left_line, left_local_margin), (right_line, right_local_margin)) = extract!{ win => left, right in [left, right] };

                spaces.push(
                    resolve(
                        (
                            left_line.right_margin() +
                            right_line.left_margin()
                        ).saturating_sub(left_local_margin + right_local_margin)
                    )
                );
            }

            spaces
        }
    }


    pub const LINE_LEN: u16 = 3 * SPACING / 2;

    impl<'c, Data: 'c> Diagram<data::Selectable<Data>>
    where Data: behavior::SimpleView + std::fmt::Debug {

        pub fn view(&mut self, render: Render) -> iced::Element<Message> {
            let level = self.level() - 1;// NOTE: Since `ViewIndex::Leveled` is shifted left.
            let input_count = self.input_count();

            let prev =
            self.prev
                .view(render);

            let mut cells =
            self.cells
                .iter_mut_timeless_indices()
                .map(|(index, data)| (ViewIndex::Leveled { level, path: vec![index] }, data))
                .collect_vec();

            let mut line_system = Spacer::new(input_count);

            let mut parts = vec![];

            while !cells.is_empty() {
                // let (_, _, this) = Self::view_diagram(dbg![&mut cells], &mut line_system, render);
                let (_, _, this) = Self::view_diagram(&mut cells, &mut line_system, render);

                parts.push(this);
            }

            parts.reverse();

            let parts =
            if parts.is_empty() {
                view_line(LINE_LEN)

            } else {
                iced::Row::with_children(parts)
                    .spacing(SPACING * 2)
                    .into()
            };

            iced::Element::from(
                iced::Row::new()
                    .spacing(SPACING * 3)
                    .push(prev)
                    .push(parts)
            ).explain(color![255, 0, 0])
        }

        fn view_diagram(
            cells: &mut Vec<(ViewIndex, &'c mut Cell<data::Selectable<Data>>)>,
            outer_widths: &mut Spacer,
            render: Render,
        ) -> ((u16, u16), Vec<u16>, iced::Element<'c, Message>)
        {
            if let Some((index, this_cell)) = cells.pop() {
                let (level, path) = extract![index => level, path in ViewIndex::Leveled { level, path }];

                let input_count = this_cell.face().ends.len();


                let mut upstream = vec![];
                let mut upstream_widths = vec![];

                let mut line_len = LINE_LEN;
                let mut free_ends = vec![];


                // NOTE: Render subdiagrams going up the input lines.
                //
                // NOTE: Reverse as to render in the right order and enable the leftmost cells
                // to be `before` the rightmost.
                //
                for i in (0 .. input_count).rev() {
                    // Find the cell which runs into the next input...
                    //
                    let is_connected =
                    cells
                        .last()
                        .map(|(_, cell)| cell.face().fill == this_cell.face().ends[i])
                        .unwrap_or(false);

                    // ...and connect it.
                    //
                    if is_connected {
                        let ((height, width), widths, subdiagram) = Self::view_diagram(cells, outer_widths, render);

                        upstream.push(subdiagram);
                        line_len = line_len.max(height);

                        upstream_widths.extend(widths);

                    } else {
                        free_ends.push(i);

                        let outer_width = outer_widths[i].width();
                        upstream_widths.push(LINE_WIDTH.max(outer_width));
                    }
                }

                // NOTE: Insert the free lines. Upstream must be temporarily reordered.
                //
                {
                    upstream.reverse();

                    for i in free_ends.into_iter().rev() {// NOTE: Reverse due to growth of `upstream.len()`.
                        upstream.insert(i, view_line(line_len));
                    }

                    upstream.reverse();
                }


                let ((this_height, mut this_width), this_cell) = this_cell.view(level, path, outer_widths, render);


                // for (i, space) in line_system.spaces_around(&upstream_widths).into_iter().enumerate().rev() {
                //     this_width += space;

                //     #[cfg(debug_assertions)] upstream.insert(i, spacer(space, i % 2 == 0));
                //     #[cfg(not(debug_assertions))] upstream.insert(i, spacer(space));
                // }

                upstream.reverse();
                upstream_widths.reverse();
                //
                // NOTE: Reverse back


                for &width in &upstream_widths {
                    this_width += width;
                }

                outer_widths.group(this_width, input_count);


                let upstream =
                iced::Row::with_children(upstream)
                    .align_items(iced::Align::Center)
                    .spacing(PADDING)
                    .padding(0)
                    .width(this_width.into());


                let diagram =
                iced::Column::new()
                    .push(
                        upstream
                    )
                    .push(
                        this_cell
                    )
                    .push(view_line(LINE_LEN))
                    .align_items(iced::Align::Center)
                    .width(this_width.into())
                    .into();


                ((line_len + LINE_LEN + this_height, this_width), upstream_widths, diagram)

            } else {
                ((LINE_LEN, LINE_WIDTH), vec![], view_line(LINE_LEN))
            }
        }
    }

    impl<Data> Cell<data::Selectable<Data>>
    where Data: behavior::SimpleView + std::fmt::Debug {
        fn view(
            &mut self,
            level: usize,
            path: Vec<TimelessIndex>,
            outer_widths: &mut Spacer,
            render: Render,
        ) -> ((u16, u16), iced::Element<Message>) {

            let mut content_height = 0;
            let mut content_width = 0;

            let content =
            if let Some(content) = &mut self.content {
                let mut inner_cells =
                content
                    .iter_mut_timeless_indices()
                    .map(|(index, data)| {
                        let mut path = path.clone();
                        path.push(index);

                        (
                            ViewIndex::Leveled { level, path },
                            data,
                        )
                    })
                    .collect();


                let mut inner_widths = outer_widths.phantom();
                let ((height, width), widths, diag) = Diagram::view_diagram(&mut inner_cells, &mut inner_widths, render);

                outer_widths.absorb(&inner_widths);

                content_height = height;
                content_width = width + 2 * PADDING;

                let diag = pad(diag);

                #[cfg(debug_assertions)] {
                    Some(diag.explain(color![255, 0, 0]))
                }

                #[cfg(not(debug_assertions))] {
                    Some(diag)
                }

            } else {
                None
            };

            let ((data_height, data_width), data) = self.meta.data.view_cell(ViewIndex::Leveled { level, path }, content_width, content, render);

            let width = content_width.max(data_width);

            ((content_height + data_height, width), data)
        }
    }
}
