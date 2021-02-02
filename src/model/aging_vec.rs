use std::{
    fmt,
    ops,
};


#[derive(Debug)]
pub enum IndexError<Age: Ord = u64> {
    OutOfBounds(Index<Age>),
}



pub struct Aging<X, Age: Ord = u64> {
    val: X,
    age: Age,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Index<Age: Ord = u64> {
    pos: usize,
    age: Age,
}


pub struct AgingVec<X, Age: Ord = u64> {
    rows: Vec<Aging<X, Age>>,

    generation: Age,
}


impl<X, Age: Ord + Default> Default for AgingVec<X, Age> {
    fn default() -> Self {
        Self {
            rows: fill![],
            generation: fill![],
        }
    }
}

impl<X, Age: Ord + Default> AgingVec<X, Age> {
    pub fn new() -> Self {
        fill![]
    }
}

impl<X, Age: Ord + Copy> AgingVec<X, Age> {
    pub fn push(&mut self, val: X) {
        let val = unsafe { Aging::from_raw_parts(val, self.generation) };

        self.rows.push(val)
    }
}

impl<X, Age: Ord> AgingVec<X, Age> {
    fn absolute_index(&self, index: Index<Age>) -> Option<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, val)| val.age <= index.age)
            .nth(index.pos)
            .map(|(idx, _)| idx)
    }
}

impl<X, Age: Ord + Copy + num::One + std::ops::AddAssign> AgingVec<X, Age> {
    pub fn insert(&mut self, index: usize, val: X) {
        self.generation += Age::one();

        let val = unsafe { Aging::from_raw_parts(val, self.generation) };

        self.rows.insert(index, val)
    }
}

impl<X, Age: Ord + Copy + num::One + std::ops::AddAssign + fmt::Debug> AgingVec<X, Age> {
    pub fn insert_relative(&mut self, index: Index<Age>, val: X) {
        self.try_insert_relative(index, val).expect(&format!["index out of bounds: {}", index]);
    }

    pub fn try_insert_relative(&mut self, index: Index<Age>, val: X) -> Result<(), IndexError<Age>> {
        let index = self.absolute_index(index).ok_or(IndexError::OutOfBounds(index))?;

        self.insert(index, val);

        Ok(())
    }
}


impl<X, Age: Ord> Aging<X, Age> {
    unsafe fn from_raw_parts(val: X, age: Age) -> Self {
        Self { val, age }
    }
}

impl<Age: Ord> ops::Add for Index<Age> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let pos = self.pos + other.pos;
        let age = self.age.max(other.age);

        Self { pos, age }
    }
}

impl<Age: Ord, N: Into<usize>> ops::Add<N> for Index<Age> {
    type Output = Self;

    fn add(self, other: N) -> Self::Output {
        let pos = self.pos + other.into();
        let age = self.age;

        Self { pos, age }
    }
}
impl<Age: Ord, N: Into<usize>> ops::AddAssign<N> for Index<Age> {
    fn add_assign(&mut self, other: N) {
        self.pos += other.into();
    }
}

impl<Age: Ord + fmt::Debug> fmt::Display for Index<Age> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write![fmt, "{}@{:?}", self.pos, self.age]
    }
}
