use std::{ ops, fmt };
use std::convert::TryFrom;




#[derive(Debug, Copy, Clone)]
pub struct TimeSpan {
    pub start: usize,
    pub end: Option<usize>,
}

#[derive(Debug, Copy, Clone)]
pub struct Trace {
    pub span: TimeSpan,

    pub loc: Option<usize>,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct TimedIndex {
    pub(in super) traces: Vec<Trace>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Subtree {
    // TODO
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IndexError {
    EmptyTracesForIndex,

    InvalidIndex(TimedIndex),
}


impl Subtree {
    pub fn iter(&self) -> impl Iterator<Item = &TimedIndex> {
        todo![];

        vec![].into_iter()
    }

    pub fn depth(&self) -> usize {
        todo!()
    }
}



impl From<usize> for TimeSpan {
    fn from(start: usize) -> Self {
        Self {
            start,
            end: None,
        }
    }
}
impl TryFrom<Vec<Trace>> for TimedIndex {
    type Error = IndexError;

    fn try_from(traces: Vec<Trace>) -> Result<Self, Self::Error> {
        if traces.is_empty() {
            Err(IndexError::EmptyTracesForIndex)

        } else {
            Ok(Self { traces })
        }
    }
}

impl From<ops::Range<usize>> for TimeSpan {
    fn from(r: ops::Range<usize>) -> Self {
        Self {
            start: r.start,
            end: Some(r.end),
        }
    }
}
impl From<ops::RangeFrom<usize>> for TimeSpan {
    fn from(r: ops::RangeFrom<usize>) -> Self {
        Self {
            start: r.start,
            end: None,
        }
    }
}

impl From<TimedIndex> for Subtree {
    fn from(index: TimedIndex) -> Self {
        Self {
            // TODO
        }
    }
}


impl Eq for TimedIndex {}
impl PartialEq for TimedIndex {
    fn eq(&self, other: &Self) -> bool {
        self.traces.starts_with(&other.traces) ||
        other.traces.starts_with(&self.traces)
    }
}

impl Eq for Trace {}
impl PartialEq for Trace {
    fn eq(&self, other: &Self) -> bool {
        self.loc == other.loc &&
        self.index == other.index &&

        self.span.start == other.span.start && (
            self.span.end == other.span.end ||
            self.span.end.or(other.span.end).is_none()
        )
    }
}

impl ops::Deref for TimedIndex {
    type Target = Trace;

    fn deref(&self) -> &Self::Target {
        self.traces.last().unwrap()
    }
}

impl fmt::Display for TimedIndex {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write![fmt, "{}", self.traces.last().unwrap()]
    }
}
impl fmt::Display for Trace {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(parent) = self.loc {
            write![fmt, "{}/{}", parent, self.index]

        } else {
            write![fmt, "/{}", self.index]
        }
    }
}
