use std::ops;

use crate::components::opetope::viewing::Message;

use super::diagram::viewing::{
    view_line,

    LINE_LEN,
};


pub enum Space<'e> {
    Group { min_width: u16, inner: Vec<Self> },

    End { min_width: u16, element: iced::Element<'e, Message> },
}

pub struct Spacer<'e> {
    spaces: Vec<Space<'e>>,
}


impl Space<'_> {
    pub fn width(&self) -> u16 {
        match self {
            Self::End { min_width, .. } =>
                *min_width,

            Self::Group { min_width, inner } =>
                inner
                    .iter()
                    .map(Self::width)
                    .sum::<u16>()
                    .max(*min_width),
        }
    }

    pub fn width_mut(&mut self) -> &mut u16 {
        match self {
            Self::End { min_width, .. } |  Self::Group { min_width, .. } =>
                min_width,
        }
    }
}

impl Spacer<'_> {
    pub fn group(&mut self, min_width: u16, count: usize) {
        let inner = self.spaces.split_off(self.spaces.len() - count);

        self.spaces.insert(0, Space::Group { min_width, inner });
    }

    pub fn phantom<'a>(&self) -> Spacer<'a> {
        let mut spaces = vec![];

        for space in &self.spaces {
            spaces.push(Space::End {
                min_width: space.width(),
                element: view_line(LINE_LEN),
            });
        }

        Spacer {
            spaces,
        }
    }

    pub fn new<'a>(count: usize) -> Spacer<'a> {
        let mut spaces = vec![];

        for _ in 0 .. count {
            spaces.push(Space::End {
                min_width: 0,
                element: view_line(LINE_LEN),
            })
        }

        Spacer {
            spaces,
        }
    }

    pub fn absorb(&mut self, other: &Self) {
        for (this, other) in self.spaces.iter_mut().zip(&other.spaces) {
            *this.width_mut() = this.width().max(other.width());
        }
    }
}

impl<'e> ops::Index<usize> for Spacer<'e> {
    type Output = Space<'e>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.spaces[ self.spaces.len() - 1 - index ]
    }
}
impl ops::IndexMut<usize> for Spacer<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.spaces.len();

        &mut self.spaces[len - 1 - index]
    }
}
