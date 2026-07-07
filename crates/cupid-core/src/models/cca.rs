#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CcaIdx(pub i32);

/// A CCA (co-curricular activity) that owns positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cca {
    pub id: CcaIdx,
    pub name: String,
}

impl Cca {
    pub fn new(id: i32, name: impl Into<String>) -> Self {
        Cca {
            id: CcaIdx(id),
            name: name.into(),
        }
    }
}
