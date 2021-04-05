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
        Self::get_helper(&self.cells, path)
    }

    fn get_helper<'s>(global: &'s TracingVec<Cell<Data>>, path: &[TimelessIndex]) -> Option<&'s Cell<Data>> {
        let mut acc =
        global
            .get(*path.first()?)
            .ok()?;

        for &seg in &path[1..] {
            acc = acc.get(seg)?;
        }

        Some(acc)
    }

    pub(in super) fn get_mut(&mut self, path: &[TimelessIndex]) -> Option<&mut Cell<Data>> {
        Self::get_mut_helper(&mut self.cells, path)
    }

    fn get_mut_helper<'s>(global: &'s mut TracingVec<Cell<Data>>, path: &[TimelessIndex]) -> Option<&'s mut Cell<Data>> {
        let mut acc =
        global
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

    #[allow(dead_code)]
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
        Self::cell_space_mut_helper(&mut self.cells, path)
    }

    fn cell_space_mut_helper<'s>(global: &'s mut TracingVec<Cell<Data>>, path: &[TimelessIndex]) -> &'s mut TracingVec<Cell<Data>> {
        if Self::get_helper(global, path).is_some() {
            let owner =
            Self::get_mut_helper(global, path)
                .unwrap();

            owner
                .content
                    .as_mut()
                    .unwrap()

        } else {
            global
        }
    }


    #[allow(dead_code)]
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

    // NOTE: Expects whole path to the cell.
    //
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

    // NOTE: Expects whole path to the cell.
    //
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

            self.select_unchecked(cell)?;// TODO: Select all cells between `cell` and the boundary of `self.selected_cells()`.

            Ok(self.selected_cells())

        } else if self.level() > cell.level() {
            self.unselect_all(self.level());

            self.prev
                .select(cell)

        } else {
            Err(Error::TooMuchDepth(cell.level()))
        }
    }

    pub(in super) fn select_unchecked(&mut self, cell: &ViewIndex) -> Result<(), Error> {
        self.get_mut(&cell.path())
            .ok_or(Error::NoSuchCell(cell.clone()))?
            .data_mut()
            .select();

        Ok(())
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
            Some(sel)

        } else {
            self.selected_cells_no_prev()
        }
    }

    fn selected_cells_no_prev(&self) -> Option<Selection> {
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

impl<Data> Diagram<data::Selectable<Data>> {
    pub fn retain_selected(self) -> Result<Option<Tail<data::Selectable<Data>>>, Error> {
        use super::utils::CellCoordinator;

        if let Some(sel) = self.selected_cells_no_prev() {
            self.check_form_tree(&sel.as_paths())?;
            let level = self.level() - 1;

            let Self { prev, mut cells } = self;
            let cell_space = Self::cell_space_mut_helper(&mut cells, &sel.common_path());

            let selected_indices =
            sel
                .as_paths()
                .into_iter()
                .map(|mut path| path.pop().unwrap())
                .collect_vec();

            let other_cells =
            cell_space
                .timeless_indices()
                .filter(|index| !selected_indices.contains(index));

            for other_cell in other_cells {
                cell_space.remove(other_cell);// TODO: Move out of `cell_space` into `cells`// FIXME: Messes up indices
            }

            let coordinator = CellCoordinator::new(level, cell_space);

            let mut walker = coordinator.walk_breadth();

            let mut select_prev = |mut prev: Tail<_>, _, cell: &mut MetaCell<data::Selectable<Data>>| {
                prev.select(&cell.face().fill).unwrap();

                ((), prev)
            };

            let mut retain_prev = |mut prev: Tail<_>, _| {
                take_mut::take(
                    &mut prev,
                    |prev|
                    prev.retain_selected()
                        .unwrap()
                        .unwrap()
                );

                prev
            };

            walker
                .on_node(&mut select_prev)
                .on_flatten(&mut retain_prev);

            let prev = walker.walk(prev);

            let mut this =
            Self {
                prev,
                cells,
            };

            this.unselect_all(0);

            Ok(Some(Tail::Diagram(Box::new(
                this
            ))))

        } else {
            self.prev.retain_selected()
        }
    }
}


// IMPL: Accessing
//
impl<Data> Cell<Data> {
    pub const fn is_end(&self) -> bool {
        !self.is_group()
    }

    pub const fn is_group(&self) -> bool {
        self.content.is_some()
    }


    pub const fn face(&self) -> &Face {
        &self.meta.face
    }

    fn face_mut(&mut self) -> &mut Face {
        &mut self.meta.face
    }

    pub fn input_count(&self) -> usize {
        self.face().ends.len()
    }


    pub const fn data(&self) -> &Data {
        &self.meta.data
    }

    fn data_mut(&mut self) -> &mut Data {
        &mut self.meta.data
    }


    pub fn get(&self, seg: TimelessIndex) -> Option<&Self> {
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
    #[allow(dead_code)]
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


#[allow(dead_code)]
pub mod viewing {
    use crate::components::opetope::{
        utils::{ Spacer, CellCoordinator, routines, LINE_LEN },
        viewing::Message,

        diagram::*,
    };

    use crate::behavior;
    use crate::model::Render;

    use crate::styles::container::cell::{ SPACING };



    impl<'c, Data: 'c> Diagram<data::Selectable<Data>>
    where Data: behavior::SimpleView {
        pub fn view(&mut self, render: Render) -> iced::Element<Message> {
            let level = self.level() - 1;// NOTE: Since `ViewIndex::Leveled` is shifted left.

            let prev =
            self.prev
                .view(render);


            let mut cells =
            self.cells
                .iter_mut_timeless_indices()
                .collect_vec();

            let mut parts = vec![];

            while !cells.is_empty() {
                let coordinator = CellCoordinator::collect_from(level, vec![], &mut cells);

                parts.push(coordinator.view(render));
            }

            parts.reverse();

            let parts =
            if parts.is_empty() {
                routines::view_line(LINE_LEN)

            } else {
                iced::Row::with_children(parts)
                    .spacing(SPACING * 2)
                    .into()
            };

            if cfg![debug_assertions] {
                iced::Element::from(
                    iced::Row::new()
                        .spacing(SPACING * 3)
                        .push(prev)
                        .push(parts)
                ).explain(color![255, 0, 0])

            } else {
                iced::Row::new()
                    .spacing(SPACING * 3)
                    .push(prev)
                    .push(parts)
                    .into()
            }
        }
    }


    impl<Data> MetaCell<data::Selectable<Data>>
    where Data: behavior::SimpleView {
        pub fn view<'s>(
            &'s mut self,

            index: ViewIndex,
            content: Option<iced::Element<'s, Message>>,

            mut spacer: Spacer,

            render: Render,
        ) -> (u16, Spacer, iced::Element<'s, Message>) {

            let ((data_height, data_width), data) = self.data.view_cell(index, spacer.width(), content, render);

            spacer.grow(data_width);

            (data_height, spacer, data)
        }
    }
}
