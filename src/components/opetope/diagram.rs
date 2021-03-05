use itertools::Itertools;
use crate::utils::{ EncapsulateIter, ProjectIter };

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

                    let wrap = self.wrap_extrusion(wrap, &ends, fill.clone());

                    Interaction::InPrevious {
                        wrap,
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
    sprout(&mut self, index: &ViewIndex, end: Data, wrap: Data)
        in prev {
            self
                .prev
                .sprout(index, end, wrap.clone())
                .map(|inter| {
                    let (fill, end) = extract![inter => group, end in Interaction::Here { action: Action::Sprout { group, end } }];

                    self.replace_line(&index.path(), &fill.path()).unwrap();

                    let wrap = self.wrap_sprout(wrap.into(), &fill, end.clone());

                    Interaction::InPrevious {
                        wrap,
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
                let face = cell.face.clone();

                let end = MetaCell {
                    data: end.into(),

                    face,
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
                .map(|(_, cell)| &cell.face)
        );

        let mut inner = vec![];

        let index =
        cell_space
            .try_replace_with(cells, |cells| {
                let content = TracingVec::from(cells);

                inner = content.timeless_indices().collect();

                MetaCell { data, face, content: Some(content) }
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

    fn wrap_sprout(&mut self, data: Data, old_end_line: &ViewIndex, sprouted_line: ViewIndex) -> ViewIndex {
        let mut cells_below =
        self.cells
            .iter_indices();

        let mut cells_with_fill_before = cells_below
            .take_while_ref(|(_, cell)|
                !cell.face.ends.contains(old_end_line)
            )
            .encapsulate();

        // NOTE: `cells_below` are now the downstream cells.
        // NOTE: `cells_with_fill_before` are the upstream cells and the parallel cells.

        let cells_with_end_after = cells_with_fill_before
            .take_while_ref(|(_, cell)| !self.has_fill_before(cell, old_end_line))
            .encapsulate();

        // NOTE: `cells_with_fill_before` are now cells between the downstream cells and the
        // preceding cells.

        // NOTE: `cells_with_end_after` are the upstream cells and the parallel cells running
        // into the ends to the right of `old_end_line`.

        let mut cells_below = cells_below.proj_l();
        let mut cells_with_end_after = cells_with_end_after.proj_l();

        let cells_with_fill_before = cells_with_fill_before.proj_l();

        let wrap = MetaCell {
            data,
            face: Face {
                ends: vec![sprouted_line],
                fill: old_end_line.clone(),
            },

            content: None,
        };

        dbg![&cells_with_end_after];

        let inserted =// FIXME: Multiple sprouting messes up
        if let Some(preceding) = dbg![cells_with_fill_before].last() {
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
        let mut upstream_outputs = old_fill_lines.to_vec();

        upstream_outputs.sort_by(|a, b| {
            if self.prev.is_before(a, b) {
                std::cmp::Ordering::Less

            } else {
                std::cmp::Ordering::Greater
            }
        });

        let lowest_fill_line = upstream_outputs.pop().unwrap();

        let mut cells_above =
        self.cells
            .iter_indices()
            .filter(|(_, cell)| lowest_fill_line == cell.face.fill)
            .proj_l();

        let wrap = MetaCell {
            data,
            face: Face {
                ends: old_fill_lines.to_vec(),
                fill: created_line,
            },

            content: None,
        };

        let index =
        if let Some(cell_above) = cells_above.next() {
            self.cells.insert_after(cell_above, wrap)
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

    fn wrap_split(&mut self, wrap_data: Data, split_line: &ViewIndex) -> ViewIndex {
        todo!()
    }
}

// IMPL: Utils
//
impl<Data> Diagram<Data> {
    fn replace_line(&mut self, line: &[TimelessIndex], new: &[TimelessIndex]) -> Result<(), Error> {
        for cell in self.cells.iter_mut() {
            cell.face.fill.subst_prefix(line, new);

            cell.face
                .ends
                .iter_mut()
                .for_each(|end| end.subst_prefix(line, new));

            cell.replace_line(line, new)?;
        }

        Ok(())
    }


    fn cells_with_end(&self, end: &ViewIndex) -> Vec<TimedIndex> {
        let mut cells =
        self.cells
            .iter_indices()
            .filter(|(_, cell)| cell.face.ends.contains(end))
            .collect();

        self.sort(&mut cells);

        cells
            .into_iter()
            .map(|(index, _)| index)
            .collect()
    }

    fn cells_after_end(&self, end: &ViewIndex) -> Vec<TimedIndex> {
        let mut cells =
        self.cells
            .iter_indices()
            .filter(|(_, cell)|
                cell.face
                    .ends
                    .iter()
                    .all(|e| self.prev.is_before(end, e))
            )
            .collect();

        self.sort(&mut cells);

        cells
            .into_iter()
            .map(|(index, _)| index)
            .collect()
    }


    fn has_fill_before(&self, cell: &MetaCell<Data>, end: &ViewIndex) -> bool {
        dbg![self.prev.is_before(dbg![&cell.face.fill], dbg![end])]
    }

    fn has_ends_before(&self, cell: &MetaCell<Data>, end: &ViewIndex) -> bool {
        cell.face
            .ends
            .iter()
            .all(|e| self.prev.is_before(e, end))
    }


    // FIXME: Trait bounds
    //
    fn choose_boundary<I>(mut cells: I, edit: Edit) -> Option<I::Item> where I: Iterator, I::Item: std::fmt::Debug + Clone {
        match edit {
            Edit::Sprout =>
                dbg![cells.last()],

            Edit::Extrude =>
                cells.next(),
        }
    }

    fn sort<I>(&self, cells: &mut Vec<(I, &MetaCell<Data>)>) {
        cells
            .sort_unstable_by(|(_, a), (_, b)| {
                let ends_before = a
                    .face
                    .ends
                    .iter()
                    .cartesian_product(
                        b
                        .face
                        .ends
                        .iter()
                    )
                    .all(|(a, b)| self.prev.is_before(a, b));

                if ends_before {
                    std::cmp::Ordering::Less

                } else {
                    std::cmp::Ordering::Greater
                }
            });
    }


    fn cell_space(&self, path: &[TimelessIndex]) -> &TracingVec<MetaCell<Data>> {
        if let Some(owner) = self.get(path) {
            owner
                .content
                    .as_ref()
                    .unwrap()

        } else {
            &self.cells
        }
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
        let outputs: Vec<_> =
        self.cells
            .iter()
            .map(|cell| &cell.face.fill)
            .collect();

        self.cells
            .iter()
            .map(|cell| cell.face.ends.to_vec())
            .flatten()
            .filter(|input| !outputs.contains(&input))
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

        let inputs: Vec<_> =
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
}

/// Common opetope interface
///
impl<Data> Diagram<Data> {
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

    fn remove_common_prefix<X>(path_a: &mut Vec<X>, path_b: &mut Vec<X>) -> Vec<X> where X: PartialEq {
        path_a.reverse();
        path_b.reverse();

        let mut common = vec![];

        while !path_a.is_empty() && !path_b.is_empty() && path_a[0] == path_b[0] {
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
    fn valid_top_level(&self, index: &ViewIndex) -> Result<TimelessIndex, Error> {
        Ok(
            self.valid_level(index)?
                .first()
                .copied()
                .unwrap()
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

            cell.data
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

// IMPL: Utils
//
impl<Data> MetaCell<Data> {
    fn replace_line(&mut self, line: &[TimelessIndex], new: &[TimelessIndex]) -> Result<(), Error> {
        if let Some(content) = &mut self.content {
            for cell in content.iter_mut() {
                cell.face.fill.subst_prefix(line, new);

                cell.face
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
    use crate::components::opetope::viewing::Message;

    use crate::behavior;

    use std::iter;
    use itertools::Itertools;


    const SPACING: u16 = crate::styles::container::cell::SPACING;
    const PADDING: u16 = crate::styles::container::PADDING;

    impl<'c, Data: 'c> Diagram<data::Selectable<Data>>
    where Data: behavior::SimpleView {

        pub fn view(&mut self) -> iced::Element<Message> {
            let mut input_separations =
            self.all_inputs()
                .into_iter()
                .map(|_| SPACING);

            let level = self.level() - 1;// NOTE: Since `ViewIndex::Leveled` is shifted left.

            let prev =
            self.prev
                .view();

            let mut cells: Vec<_> =
            self.cells
                .iter_mut_timeless_indices()
                .map(|(index, data)| (ViewIndex::Leveled { level, path: vec![index] }, data))
                .collect();

            let (_, this) = Self::view_diagram(&mut input_separations, &mut cells);

            iced::Row::new()
                .spacing(SPACING * 2)
                .push(prev)
                .push(this)
                .into()
        }

        fn view_diagram(
            input_separations: &mut impl Iterator<Item = u16>,
            cells: &mut Vec<(ViewIndex, &'c mut MetaCell<data::Selectable<Data>>)>,
        ) -> (Vec<u16>, iced::Element<'c, Message>)
        {
            if let Some((index, this_cell)) = cells.pop() {
                let (level, path) = extract![index => level, path in ViewIndex::Leveled { level, path }];

                let mut upstream = vec![];
                let mut widths = vec![SPACING];

                let input_count = this_cell.face.ends.len();

                // NOTE: Render subdiagrams going up the input lines.
                //
                for i in 0 .. input_count {
                    // If the next cell runs into the next input...
                    //
                    let is_connected =
                    if let Some((_, cell)) = cells.last() {
                        this_cell.face.ends[i] == cell.face.fill

                    } else {
                        false
                    };

                    // ...connect it.
                    //
                    if is_connected {
                        let (margins, subdiagram) = Self::view_diagram(input_separations, cells);

                        let width =
                        margins
                            .into_iter()
                            .fold(0, |acc, x| acc + x);

                        upstream.push(Some(subdiagram));
                        widths.push(width - 2 * SPACING);// NOTE: `SPACING` is always added as a margin.

                    } else {
                        upstream.push(None);
                        widths.push(input_separations.next().unwrap());
                    }
                }

                widths.push(SPACING);

                let mut min_margins = vec![];

                // NOTE: Calculate minimal margins of input lines.
                //
                for i in 0 .. input_count + 1 {
                    let margin = widths[i].max(widths[i + 1]);

                    min_margins.push(margin);
                }

                let (margins, this_cell) = this_cell.view(min_margins, level, path);

                let subdiagrams = Self::view_subdiagrams(&margins, &widths, upstream);

                let lines = Self::view_lines(&margins);

                let diagram =
                iced::Column::new()
                    .push(
                        iced::Row::with_children(subdiagrams).align_items(iced::Align::Center)
                    )
                    .push(
                        iced::Row::with_children(lines).align_items(iced::Align::Center)
                    )
                    .push(
                        this_cell
                    )
                    .into();

                (margins, diagram)

            } else {
                (vec![0], Self::view_line())
            }
        }

        fn view_subdiagrams(margins: &[u16], widths: &[u16], upstream: Vec<Option<iced::Element<'c, Message>>>) -> Vec<iced::Element<'c, Message>> {
            let upstream_spacers =
            margins
                .iter()
                .copied()
                .zip(widths.iter().copied())
                .map(|(margin, width)| margin - width)
                .map(|width| iced::Space::with_width(iced::Length::Units(width)).into());


            upstream_spacers
                .interleave(
                    upstream
                        .into_iter()
                        .map(|subdiagram| subdiagram.unwrap_or_else(|| iced::Space::with_height(0.into()).into()))
                )
                .collect()
        }

        fn view_lines(margins: &[u16]) -> Vec<iced::Element<'static, Message>> {
            let line_spacers =
            margins
                .iter()
                .copied()
                .map(|width| iced::Space::with_width(iced::Length::Units(width)).into());


            line_spacers
                .interleave_shortest(
                    iter::repeat_with(|| Self::view_line())
                        .take(margins.len() - 1)
                )
                .collect()
            //
            // TODO: Account for lines' width.
        }

        fn view_line() -> iced::Element<'static, Message> {
            iced::Container::new(
                iced::Space::new(
                    iced::Length::Units(3),
                    iced::Length::Units(30),// TODO: Figure out how to make it responsive.
                )
            )
            .style(crate::styles::container::LINE)
            .into()
        }
    }

    impl<Data> MetaCell<data::Selectable<Data>>
    where Data: behavior::SimpleView {
        fn view(&mut self, line_margins: Vec<u16>, level: usize, path: Vec<TimelessIndex>) -> (Vec<u16>, iced::Element<Message>) {

            let (margins, content) =
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

                let (mut actual_margins, content) = Diagram::view_diagram(&mut line_margins.into_iter(), &mut inner_cells);

                actual_margins[0] += PADDING;
                *actual_margins.last_mut().unwrap() += PADDING;

                (actual_margins, Some(content))

            } else {
                (line_margins, None)
            };

            (margins, self.data.view_cell(ViewIndex::Leveled { level, path }, content))
        }
    }
}
