use std::convert::TryInto;


mod index;

use index::*;
pub use index::{ IndexError, TimedIndex, Subtree };

type IResult<X> = Result<X, IndexError>;



#[derive(Debug, Clone)]
pub struct TTree<X> {
    mem: Vec<(TimedIndex, X)>,

    pseudotime: usize,
}



impl<X> Default for TTree<X> {
    fn default() -> Self {
        Self::new()
    }
}

impl<X> TTree<X> {
    pub fn new() -> Self {
        Self {
            mem: vec![],
            pseudotime: 0,
        }
    }
}

impl<X> From<Vec<X>> for TTree<X> {
    fn from(v: Vec<X>) -> Self {
        let mem = v
            .into_iter()
            .enumerate()
            .map(|(index, val)| (
                vec![Trace {
                    span: TimeSpan::from(0),
                    loc: None,
                    index
                }].try_into().unwrap(),
                val,
            ))
            .collect();

        Self {
            mem,
            pseudotime: 0,
        }
    }
}


/// Mutations
///
impl<X> TTree<X> {
    pub fn push(&mut self, val: X) -> TimedIndex {
        let index = self.last_pos();

        let index: TimedIndex = vec![
            Trace {
                span: (self.pseudotime..).into(),
                loc: None,
                index,
            }
        ].try_into().unwrap();

        self.mem.push((index.clone(), val));

        index
    }

    pub fn insert_child(&mut self, parent: &TimedIndex, val: X) -> IResult<TimedIndex> {
        let pos = self.pos(parent)?;

        todo!()
    }
}

/// Accessing
///

impl<X> TTree<X> {
    pub fn get(&self, index: &TimedIndex) -> IResult<&X> {
        todo![]
    }

    pub fn get_mut(&mut self, index: &TimedIndex) -> IResult<&mut X> {
        todo![]
    }


    pub fn depth(&self) -> usize {
        let mut depth = 0;

        todo![];

        depth
    }
}

/// Iteration
///
impl<X> TTree<X> {
    pub fn iter(&self) -> impl Iterator<Item = &X> {
        self.mem
            .iter()
            .filter(|(index, _)| index.loc.is_none())
            .map(|(_, val)| val)
    }

    pub fn indices(&self) -> impl Iterator<Item = &TimedIndex> {
        self.mem
            .iter()
            .map(|(index, _)| index)
            .filter(|index| index.loc.is_none())
    }
}

/// Utils
///
impl<X> TTree<X> {
    fn last_pos(&self) -> usize {
        self.indices()
            .last()
            .map(|index| index.index + 1)
            .unwrap_or(0)
    }

    fn pos(&self, index: &TimedIndex) -> IResult<usize> {
        self.indices()
            .enumerate()
            .filter(|(_, i)| **i == *index)
            .map(|(pos, _)| pos)
            .next()
            .ok_or(IndexError::InvalidIndex(index.clone()))
    }
}
