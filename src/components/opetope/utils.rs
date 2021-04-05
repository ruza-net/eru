use std::ops;
use itertools::Itertools;

use tracing_vec::*;
use crate::components::opetope::{
    ViewIndex,

    MetaCell,
    diagram::Cell,

    data,
    Message,
};

use crate::model::Render;
use crate::behavior;

use crate::styles::container::{ PADDING, LINE_WIDTH, cell::SPACING };

pub const LINE_LEN: u16 = 3 * SPACING / 2;



pub mod routines {
    use super::*;

    pub fn pad<'e>(e: impl Into<iced::Element<'e, Message>>) -> iced::Element<'e, Message> {
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
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spacer {
    min_width: u16,

    inner: Vec<Self>,
}
impl Default for Spacer {
    fn default() -> Self {
        Self {
            inner: vec![],
            min_width: LINE_WIDTH,
        }
    }
}
impl From<u16> for Spacer {
    fn from(min_width: u16) -> Self {
        Self {
            min_width,

            ..fill![]
        }
    }
}


/// Instance creation
///
impl Spacer {
    #[allow(dead_code)]
    pub fn new(min_width: u16, count: usize) -> Self {
        Self {
            inner: vec![ Self { min_width, ..fill![] }; count ],

            ..fill![]
        }
    }

    pub fn flatten(&self) -> Self {
        Self {
            inner: vec![],

            min_width: self.width(),
        }
    }

    pub fn group(min_width: u16, inner: Vec<Self>) -> Self {
        Self { min_width, inner }
    }
}

/// Accessing
///
impl Spacer {
    pub fn width(&self) -> u16 {
        self.inner
            .iter()
            .map(Self::width)
            .interleave_shortest(vec![PADDING; self.space_count()])
            .sum::<u16>()
            .max(self.min_width)
    }

    pub fn space_count(&self) -> usize {
        self.inner.len().saturating_sub(1)
    }
}

/// Mutation
///
impl Spacer {
    pub fn grow(&mut self, lower_bound: u16) {
        self.min_width = self.min_width.max(lower_bound);
    }

    pub fn pad(&mut self, padding: u16) {
        self.min_width += 2 * padding;// TODO: What if `self.width() > self.min_width`?
    }

    pub fn extend(&mut self, mut outer: Vec<Self>) -> Vec<Self> {
        for space in &mut self.inner {
            if space.inner.is_empty() {
                if let Some(out) = outer.pop() {
                    *space = out;

                } else {
                    break;
                }

            } else {
                outer = space.extend(outer);
            }
        }

        outer
    }
}

/// Rendering
///
impl Spacer {
    pub fn render<'e, Msg: 'e>(&self, items: &mut Vec<iced::Element<'e, Msg>>) -> iced::Element<'e, Msg> {
        if self.inner.is_empty() {
            let element = items.pop().unwrap();

            iced::Container::new(element)
                .align_x(iced::Align::Center)
                .width(self.width().into())
                .into()

        } else {
            let mut children = vec![];

            for space in self.inner.iter().rev() {
                children.push(space.render(items));

                if items.is_empty() {
                    break;
                }
            }

            iced::Container::new(
            iced::Row::with_children(children)
                .align_items(iced::Align::Center)
                .spacing(PADDING)
                )
                .align_x(iced::Align::Center)
                .width(self.width().into())
                .into()
        }
    }
}


macro_rules! index {
    ( $self:ident [ $idx:ident : $typ:ty => $out:ty ] => $precond:block => $body:expr ) => {
        impl ops::Index<$typ> for Spacer {
            type Output = $out;

            fn index(&$self, mut $idx: $typ) -> &Self::Output {
                $precond

                & $body
            }
        }

        impl ops::IndexMut<$typ> for Spacer {
            fn index_mut(&mut $self, mut $idx: $typ) -> &mut Self::Output {
                $precond

                &mut $body
            }
        }
    };
}

index! { self[index: usize => Self] => { index = self.inner.len() - 1 - index; } => self.inner[index] }
index! {
    self[span: ops::Range<usize> => [Self]] =>
    {
        let len = self.inner.len();
        span = len - span.end  .. len - span.start;
    } =>
    self.inner[span]
}



#[derive(Debug)]
pub(in super) struct CellCoordinator<'op, Data> {
    cell: &'op mut MetaCell<Data>,
    addr: ViewIndex,

    inner: Option<Box<Self>>,
    upstream: Vec<Option<Self>>,
}

pub struct BreadthWalker<'op, Data, X, State> {
    cells: Vec<CellCoordinator<'op, Data>>,

    on_node: Option<&'op mut dyn FnMut(State, ViewIndex, &mut MetaCell<Data>) -> (X, State)>,
    on_flatten: Option<&'op mut dyn FnMut(State, Vec<Option<X>>) -> State>,
}

#[allow(dead_code)]
/// Instance creation
///
impl<'op, Data> CellCoordinator<'op, Data> {
    pub fn new(level: usize, cell_space: &'op mut TracingVec<Cell<Data>>) -> CellCoordinator<'op, Data> {
        Self::collect_from(level, vec![], &mut cell_space.iter_mut_timeless_indices().collect())
    }

    pub fn collect_from(
        level: usize,
        mut path: Vec<TimelessIndex>,
        cell_space: &mut Vec<(TimelessIndex, &'op mut Cell<Data>)>,
    ) -> CellCoordinator<'op, Data>
    {
        let (addr, cell) = cell_space.pop().unwrap();
        let input_count = cell.input_count();

        let mut upstream = vec![];

        for i in (0 .. input_count).rev() {
            let is_connected =
                cell_space
                    .last()
                    .map(|(_, next)| next.face().fill == cell.face().ends[i])
                    .unwrap_or(false);

            if is_connected {
                upstream.push(Some(Self::collect_from(level, path.clone(), cell_space)));

            } else {
                upstream.push(None);
            }
        }

        path.push(addr);

        let inner =
        cell.content
            .as_mut()
            .map(|inner_space|
                Box::new(Self::collect_from(level, path.clone(), &mut inner_space.iter_mut_timeless_indices().collect()))
            );


        let cell = &mut cell.meta;
        let addr = ViewIndex::Leveled { level, path };

        Self {
            cell,
            addr,

            inner,
            upstream,
        }
    }
}

/// Accessing
///
impl<Data> CellCoordinator<'_, Data> {
    pub fn input_count(&self) -> usize {
        self.upstream
            .iter()
            .fold(0, |acc, up|
                if let Some(up) = up {
                    acc + up.input_count()
                    
                } else {
                    acc + 1
                }
            )
    }
}

/// Walking
///
impl<'op, Data> CellCoordinator<'op, Data> {
    pub fn walk_breadth<X, State>(self) -> BreadthWalker<'op, Data, X, State> {
        BreadthWalker {
            cells: vec![self],

            on_node: None,
            on_flatten: None,
        }
    }
}

impl<'op, Data, X, State> BreadthWalker<'op, Data, X, State> {
    pub fn on_node(&mut self, f: &'op mut impl FnMut(State, ViewIndex, &mut MetaCell<Data>) -> (X, State)) -> &mut Self {
        self.on_node = Some(f);
        self
    }

    pub fn on_flatten(&mut self, f: &'op mut impl FnMut(State, Vec<Option<X>>) -> State) -> &mut Self {
        self.on_flatten = Some(f);
        self
    }

    pub fn walk(self, state: State) -> State {
        let Self { cells, mut on_node, mut on_flatten } = self;

        let mut layers = vec![cells];

        let mut state = Some(state);

        while let Some(cells) = layers.pop() {
            let mut data = vec![];
            let mut new_layer = vec![];

            for cell in cells {
                let CellCoordinator { cell, addr, upstream, .. } = cell;

                data.push(
                    on_node.as_mut().map(|on_node| {
                        let (ret, new_state) = on_node(state.take().unwrap(), addr, cell);

                        state = Some(new_state);

                        ret
                    })
                );
                new_layer.extend(upstream.into_iter().filter(Option::is_some).map(Option::unwrap));
            }

            on_flatten.as_mut().map(|on_flatten| state = Some(on_flatten(state.take().unwrap(), data)));

            if !new_layer.is_empty() {
                layers.push(new_layer);
            }
        }

        state.take().unwrap()
    }
}

/// Viewing
///
impl<'op, Data: behavior::SimpleView> CellCoordinator<'op, data::Selectable<Data>> {
    pub fn view(self, render: Render) -> iced::Element<'op, Message> {
        let widths = vec![fill![]; self.input_count()];

        let without_line =
        self.render(widths, render).2;

        iced::Column::new()
            .push(without_line)
            .push(routines::view_line(LINE_LEN))
            .align_items(iced::Align::Center)
            .into()
    }

    fn render(self, mut outer_widths: Vec<Spacer>, render: Render) -> (u16, Spacer, iced::Element<'op, Message>) {
        let mut widths = vec![];
        let mut heights = vec![];


        // Rendering upstream subdiagrams
        //
        let upstream =
        self.upstream
            .into_iter()
            .map(|up|
                if let Some(up) = up {
                    let spaces = outer_widths.split_off(outer_widths.len() - up.input_count());

                    let (height, width, up) = up.render(spaces, render);

                    widths.push(width);
                    heights.push(height);

                    up

                } else {
                    widths.push(outer_widths.pop().unwrap());
                    heights.push(0);

                    routines::view_line(0)
                }
            )
            .collect_vec();


        // Inserting output lines to adjust heights
        //
        let max_height = heights.iter().max().copied().unwrap_or(0) + LINE_LEN;

        let mut upstream =
        upstream
            .into_iter()
            .zip(heights)
            .map(|(up, height)|
                iced::Column::new()
                    .push(up)
                    .push(routines::view_line(max_height - height))
                    .align_items(iced::Align::Center)
                    .into()
            )
            .collect_vec();


        // Rendering inner diagram
        //
        widths.reverse();
        let mut flat_widths = widths.iter().map(Spacer::flatten).collect();

        let (inner_height, inner_spacer, inner) =
        if let Some(inner) = self.inner {
            let (height, mut width, mut inner) = inner.render(flat_widths, render);

            inner =
            iced::Column::new()
                .push(inner)
                .push(routines::view_line(LINE_LEN))
                .align_items(iced::Align::Center)
                .into();

            width.pad(PADDING);

            (height + LINE_LEN, width, Some(routines::pad(inner)))

        } else {
            flat_widths.reverse();

            (0, Spacer::group(0, flat_widths), None)
        };


        // Rendering the whole cell
        //
        let (data_height, mut spacer, this_cell) = self.cell.view(self.addr.clone(), inner, inner_spacer, render);

        let height = max_height + inner_height + data_height;


        // Spacing the upstream subdiagrams
        //
        let upstream = spacer.render(&mut upstream);

        spacer.extend(widths);

        let diagram =
        iced::Column::new()
            .push(upstream)
            .push(this_cell)
            .align_items(iced::Align::Center)
            .into();

        (height, spacer, diagram)
    }
}
