use super::{ *, viewing::{ ViewIndex, Selection, Index } };

use std::collections::HashSet;



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

// IMPL: Editing
//
impl<Data: Clone> Diagram<Data> {
    interaction!{ extrude(&mut self, cell: &Selection, group: Data, wrap: Data)
        in prev {
            self
                .prev
                .extrude(cell, group, wrap.clone())
                .map(|inter| {
                    let fill = extract![inter => group in Interaction::Here { action: Action::Extrude { group } }];

                    let wrap = self.wrap(wrap, fill.clone());

                    Interaction::InPrevious {
                        wrap,
                        action: Action::Extrude { group: fill },
                    }
                })
                .into()
        }

        in self {
            if cell.common_path().len() > 1 {
                return EditResult::Err(Error::CannotExtrudeNestedCells(cell.clone()));
            }

            match self.group(&cell.as_paths(), group) {
                Ok(group) =>
                    EditResult::Ok(Interaction::Here {
                        action: Action::Extrude { group },
                    }),

                Err(e) => EditResult::Err(e),
            }
        }
    }

    interaction!{ sprout(&mut self, index: &ViewIndex, end: Data, wrap: Data)
        in prev {
            self
                .prev
                .sprout(index, end, wrap.clone())
                .map(|inter| {
                    let fill = extract![inter => group in Interaction::Here { action: Action::Sprout { group } }];

                    let wrap = self.wrap(wrap, fill.clone());

                    Interaction::InPrevious {
                        wrap,
                        action: Action::Sprout { group: fill },
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
                    data: end,

                    face,
                    content: None,
                };

                cell.content = Some(TracingVec::from(vec![end]));

                EditResult::Ok(Interaction::Here {
                    action: Action::Sprout { group: index.clone() },
                })

            } else {
                EditResult::Err(Error::NoSuchCell(index.clone()))
            }
        }
    }

    fn group(&mut self, cells: &[Vec<TimelessIndex>], data: Data) -> Result<ViewIndex, Error> {
        let (tail, cells) = self.check_form_tree(cells)?;

        let cell_space = self.cell_space_mut(tail);

        let face = Face::collect(
            cell_space
                .iter_timeless_indices()
                .filter(|(index, _)| cells.contains(index))
                .map(|(_, cell)| &cell.face)
        );

        let index =
        cell_space
            .try_replace_with(cells, |cells| {
                let content = Some(TracingVec::from(cells));

                MetaCell { data, face, content }
            })
            .map_err(Error::IndexError)?;

        let index =
        cell_space
            .into_timeless(index)
                .unwrap();

        let mut path = tail.to_vec();
        path.push(index);

        Ok(self.into_index(path))
    }

    fn wrap(&mut self, data: Data, fill: ViewIndex) -> ViewIndex {
        let ends =
        self.prev
            .contents_of(&fill.path())
            .unwrap();

        let cell_below = self.cell_with_end(&fill);

        let cell = MetaCell {
            data,
            face: Face { ends, fill },

            content: None,
        };

        let index =
        if let Some(index) = cell_below {
            self.cells.insert_before(index, cell);

            index

        } else {
            self.cells.push(cell);

            self.cells.last_index().unwrap()
        };

        let index =
        self.cells
            .into_timeless(index)
            .unwrap();

        self.into_index(vec![index])
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
                        &this_cell.face.ends[i] == &cell.face.fill

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
