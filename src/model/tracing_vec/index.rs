use std::fmt;



/// An index pointing to a position in a snapshot of a [`TracingVec`]. It doesn't point directly
/// to the underlying data, but retains information about the particular position and thus
/// supports a kind of "interior mutability".
///
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TimedIndex {
    pub(in super) pseudotime: usize,

    pub(in super) pos: usize,
}

/// An index pointing just to some data in a [`TracingVec`]. It doesn't retain information about
/// its movement.
///
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TimelessIndex {
    pub(in super) pos: usize,
}

/// A generic index into a tracing vector.
///
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum TracingIndex {
    Timed(TimedIndex),
    Timeless(TimelessIndex),
}


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum IndexError {
    /// Timed index points to a valid version, but to an invalid location therewithin.
    ///
    IndexOutOfBounds(TimedIndex),

    /// Timed index doesn't point to a valid version.
    ///
    VersionDoesNotExist(TimedIndex),

    /// Timeless index doesn't point to valid data.
    ///
    DataDoesNotExist(TimelessIndex),

    /// Attempted to perform an operation that requires the element to exist in the latest version.
    ///
    DataAlreadyDead(TimelessIndex),

    /// Attempted to perform a splicing operation on no indices.
    ///
    NoIndicesProvided,
}



impl TimedIndex {
    pub unsafe fn from_raw_parts(pos: usize, pseudotime: usize) -> Self {
        Self { pos, pseudotime }
    }
}

impl TimelessIndex {
    pub unsafe fn from_raw_parts(pos: usize) -> Self {
        Self { pos }
    }
}

impl fmt::Display for TimedIndex {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write![fmt, "{}@{}", self.pos, self.pseudotime]
    }
}
impl fmt::Display for TimelessIndex {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write![fmt, "{}@!", self.pos]
    }
}

impl From<TimedIndex> for TracingIndex {
    fn from(idx: TimedIndex) -> Self {
        Self::Timed(idx)
    }
}
impl From<TimelessIndex> for TracingIndex {
    fn from(idx: TimelessIndex) -> Self {
        Self::Timeless(idx)
    }
}

// AREA: Arithmetic
//
macro_rules! index_arithmetic {
    ( { } $($normal:ident [$fun:ident] & $assign:ident [$fun_assign:ident] => $op:tt),* ) => {};

    ( { $num:ty $(, $nn:ty)* } $($normal:ident [$fun:ident] & $assign:ident [$fun_assign:ident] => $op:tt),* ) => {
        index_arithmetic! {
            @fortype $num:
            $( $normal[$fun] & $assign[$fun_assign] => $op ),*
        }

        index_arithmetic! {
            { $($nn),* }
            $( $normal[$fun] & $assign[$fun_assign] => $op ),*
        }
    };

    (@fortype $num:ty : $($normal:ident [$fun:ident] & $assign:ident [$fun_assign:ident] => $op:tt),*) => {
        $(
            impl std::ops::$normal<$num> for TimedIndex {
                type Output = Self;

                fn $fun(self, other: $num) -> Self::Output {
                    let pos = self.pos $op other as usize;
                    let pseudotime = self.pseudotime;

                    Self { pos, pseudotime }
                }
            }

            impl std::ops::$assign<$num> for TimedIndex {
                fn $fun_assign(&mut self, other: $num) {
                    self.pos = self.pos $op other as usize;
                }
            }

            impl std::ops::$normal<$num> for TimelessIndex {
                type Output = Self;

                fn $fun(self, other: $num) -> Self::Output {
                    let pos = self.pos $op other as usize;

                    Self { pos }
                }
            }

            impl std::ops::$assign<$num> for TimelessIndex {
                fn $fun_assign(&mut self, other: $num) {
                    self.pos = self.pos $op other as usize;
                }
            }
        )*
    };
}

index_arithmetic! {
    { usize, u64, u32, u16, u8, isize, i64, i32, i16, i8 }
    Add[add] & AddAssign[add_assign] => +,
    Sub[sub] & SubAssign[sub_assign] => -
}
//
// END: Arithmetic
