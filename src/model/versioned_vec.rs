mod index;

pub use index::{ Index, IndexError };


// NOTE: This can be generalized to any versioned structure. It just has to provide an access API
// for additive mutations (`+mut`) and subtractive mutations (`-mut`) which upholds memory safety
// principles. Then the versioning wrapper can mutate the most recent version on additive mutation
// or create a new version on subtractive mutation.
//
/// A vector which preserves its former versions to support lazy indexing.
///
/// # Preventing Aliasing
/// This structure prevents aliasing of immutable references to its elements by maintaining many
/// versions which represent snapshots of this vector's state. Immutable references are represented
/// by indices enriched by pseudotimes. The pseudotimes determine a version of the vector which
/// is then searched for the given location.
///
pub struct VersionedVec<X> {
    versions: Vec<Vec<X>>,
}


impl<X> Default for VersionedVec<X> {
    fn default() -> Self {
        Self::new()
    }
}
impl<X> From<Vec<X>> for VersionedVec<X> {
    fn from(val: Vec<X>) -> Self {
        Self {
            versions: vec![val],
        }
    }
}

// IMPL: Initialization
//
impl<X> VersionedVec<X> {
    pub fn new() -> Self {
        Self {
            versions: fill![],
        }
    }

    pub unsafe fn from_raw_parts(versions: Vec<Vec<X>>) -> Self {
        Self { versions }
    }
}

// IMPL: Additive Mutations
//
impl<X> VersionedVec<X> {
    pub fn push(&mut self, val: X) {
        self.versions.last_mut().unwrap().push(val)
    }
}

// IMPL: Subtractive Mutations
//
impl<X: Clone> VersionedVec<X> {
    pub fn pop(&mut self) -> Option<X> {
        self
            .new_version()
            .pop()
    }

    pub fn insert(&mut self, index: Index, val: X) -> Index {
        self.try_insert(index, val).unwrap()
    }

    pub fn try_insert(&mut self, index: Index, val: X) -> Result<Index, IndexError> {
        let absolute_index = self.to_absolute(index)?;

        if absolute_index == self.latest().len() {
            self.push(val);

        } else {
            self
                .new_version()
                .insert(absolute_index, val);
        }

        let pos = absolute_index;
        let pseudotime = self.versions.len() - 1;

        Ok(Index { pos, pseudotime })
    }

    pub fn remove(&mut self, index: Index) -> Option<X> {
        self.try_remove(index).unwrap()
    }

    pub fn try_remove(&mut self, index: Index) -> Result<Option<X>, IndexError> {
        todo![]
    }
}

// IMPL: Accessing
//
impl<X> VersionedVec<X> {
    pub fn latest(&self) -> &Vec<X> {
        self.versions.last().unwrap()
    }

    pub fn last_index(&self) -> Option<Index> {
        let v = self.latest();

        if v.is_empty() {
            None

        } else {
            Some(unsafe { Index::from_raw_parts(v.len() - 1, self.pseudotime()) })
        }
    }

    pub fn oldest(&self) -> &Vec<X> {
        self.versions.first().unwrap()
    }

    pub fn contains(&self, index: Index) -> bool {
        self.get(index).is_some()
    }

    pub fn get(&self, index: Index) -> Option<&X> {
        self.versions.get(index.pseudotime).map(|v| v.get(index.pos)).flatten()
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut X> {
        self.versions.get_mut(index.pseudotime).map(|v| v.get_mut(index.pos)).flatten()
    }
}

// IMPL: Iteration
//
impl<X> VersionedVec<X> {
    pub fn indices(&self) -> impl Iterator<Item = Index> {
        let pseudotime = self.pseudotime();

        let len = self.latest().len();

        (0 .. len)
            .into_iter()
            .map(move |pos| Index { pos, pseudotime })
    }

    pub fn iter(&self) -> impl Iterator<Item = &X> {
        self
            .latest()
            .iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut X> {
        self
            .latest_mut()
            .iter_mut()
    }

    pub fn iter_indices(&self) -> impl Iterator<Item = (Index, &X)> {
        self.indices().zip(self.iter())
    }

    pub fn iter_mut_indices(&mut self) -> impl Iterator<Item = (Index, &mut X)> {
        self.indices().zip(self.iter_mut())
    }
}

// IMPL: Private Utils
//
impl<X> VersionedVec<X> {
    fn pseudotime(&self) -> usize {
        self.versions.len() - 1
    }

    fn latest_mut(&mut self) -> &mut Vec<X> {
        self.versions.last_mut().unwrap()
    }

    fn to_absolute(&self, index: Index) -> Result<usize, IndexError> {
        let Index { pos, pseudotime } = index;

        if pseudotime < self.versions.len() - 1 {
            Err(IndexError::AccessToVersionIsRestricted(index))

        } else if pseudotime >= self.versions.len() {
            Err(IndexError::VersionDoesNotExist(index))

        } else {
            Ok(pos)
        }
    }
}

impl<X: Clone> VersionedVec<X> {
    fn new_version(&mut self) -> &mut Vec<X> {
        let last_version_copy = self
            .latest()
            .clone();

        self.versions.push(last_version_copy);

        self.latest_mut()
    }
}
