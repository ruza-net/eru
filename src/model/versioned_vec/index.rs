use std::fmt;



/// An index into a [`VersionedVec`], holding both a position and a pseudotime. The position
/// determines the physical location, and the pseudotime determines which version is searched for
/// the location.
///
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Index {
    pub(in super) pos: usize,
    pub(in super) pseudotime: usize,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum IndexError {
    /// Index points to a valid version, but to an invalid location therewithin.
    ///
    IndexOutOfBounds(Index),

    /// Index doesn't point to a valid version.
    ///
    VersionDoesNotExist(Index),

    /// Index points to an older version, which means it cannot be used to mutate it nor to
    /// generate an absolute index.
    ///
    AccessToVersionIsRestricted(Index),
}


// IMPL: Unsafe API
//
impl Index {
    pub unsafe fn from_raw_parts(pos: usize, pseudotime: usize) -> Self {
        Self { pos, pseudotime }
    }
}


impl fmt::Display for Index {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write![fmt, "{}@{}", self.pos, self.pseudotime]
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
            impl std::ops::$normal<$num> for Index {
                type Output = Self;

                fn $fun(self, other: $num) -> Self::Output {
                    let pos = self.pos $op other as usize;
                    let pseudotime = self.pseudotime;

                    Self { pos, pseudotime }
                }
            }

            impl std::ops::$assign<$num> for Index {
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
