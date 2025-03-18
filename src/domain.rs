use std::fmt;
use std::rc::Rc;

//
// Simple wrapper types for match data
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TentativeMatchBit(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmedMatchBit(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexOfBestMatch(pub u64);

//
// String-containing types using Rc<str> for efficient sharing
//

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Base64EncodedString(pub Rc<str>);

impl Base64EncodedString {
    pub fn new(value: &str) -> Self {
        Self(Rc::from(value))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordId(pub Base64EncodedString);

impl RecordId {
    pub fn new(value: &str) -> Self {
        Self(Base64EncodedString::new(value))
    }
}

impl AsRef<[u8]> for RecordId {
    fn as_ref(&self) -> &[u8] {
        self.0.0.as_bytes()
    }
}

impl fmt::Display for RecordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.0)
    }
}
//
// Result data structures
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultData {
    pub confirmed_match: ConfirmedMatchBit,
    pub tentative_match: TentativeMatchBit,
    pub index_of_best_match: IndexOfBestMatch,
}

// TODO: maybe delete this `new` function?
impl ResultData {
    pub fn new(
        confirmed_match: ConfirmedMatchBit,
        tentative_match: TentativeMatchBit,
        index_of_best_match: IndexOfBestMatch,
    ) -> Self {
        Self {
            tentative_match,
            confirmed_match,
            index_of_best_match,
        }
    }

    pub fn from_strings(tentative_match: &str, confirmed_match: &str, index: &str) -> Self {
        Self {
            confirmed_match: ConfirmedMatchBit(confirmed_match.parse().unwrap_or(0)),
            tentative_match: TentativeMatchBit(tentative_match.parse().unwrap_or(0)),
            index_of_best_match: IndexOfBestMatch(index.parse().unwrap_or(0)),
        }
    }

    // Default ResultData with all zeros
    pub fn empty() -> Self {
        Self {
            confirmed_match: ConfirmedMatchBit(0),
            tentative_match: TentativeMatchBit(0),
            index_of_best_match: IndexOfBestMatch(0),
        }
    }
}

impl fmt::Display for ResultData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "match-{}-{}-{}",
            self.confirmed_match.0, self.tentative_match.0, self.index_of_best_match.0
        )
    }
}

//
// Linkage data structures
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLinkageData {
    pub share_data: Rc<Vec<ResultData>>,
}

impl ClientLinkageData {
    pub fn new(share_data: Vec<ResultData>) -> Self {
        Self {
            share_data: Rc::new(share_data),
        }
    }

    pub fn empty() -> Self {
        Self {
            share_data: Rc::new(Vec::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLinkageData {
    pub share_data: Rc<Vec<ResultData>>,
    pub record_ids: Rc<Vec<RecordId>>,
}

impl ServerLinkageData {
    pub fn new(share_data: Vec<ResultData>, record_ids: Vec<RecordId>) -> Self {
        Self {
            share_data: Rc::new(share_data),
            record_ids: Rc::new(record_ids),
        }
    }

    pub fn empty() -> Self {
        Self {
            share_data: Rc::new(Vec::new()),
            record_ids: Rc::new(Vec::new()),
        }
    }

    // Helper to get all record IDs
    pub fn ids(&self) -> &[RecordId] {
        self.record_ids.as_ref()
    }

    // Helper to get record ID at a specific index, with bounds checking
    pub fn get_id(&self, index: usize) -> Option<&RecordId> {
        if index < self.record_ids.len() {
            Some(&self.record_ids[index])
        } else {
            None
        }
    }
}

//
// Linkage ID and related types
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageID(pub Base64EncodedString);

impl LinkageID {
    pub fn new(value: &str) -> Self {
        Self(Base64EncodedString::new(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkageStatus {
    Pending,
    Complete,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageResult(pub Vec<Option<LinkageID>>);
