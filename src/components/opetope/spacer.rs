use std::ops;
use itertools::Itertools;

use crate::styles::container::{ PADDING, LINE_WIDTH };



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
