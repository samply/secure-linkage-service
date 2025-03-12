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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordId(pub Base64EncodedString);

impl RecordId {
    pub fn new(value: &str) -> Self {
        Self(Base64EncodedString::new(value))
    }
}

//
// Encryption related types
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionKey(pub Base64EncodedString);

impl EncryptionKey {
    pub fn new(value: &str) -> Self {
        Self(Base64EncodedString::new(value))
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

    // Added to support the dummy implementation
    pub fn from(value: &str) -> Self {
        Self::new(value)
    }
}

// Linkage status enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkageStatus {
    Pending,
    Complete,
    Failed,
}

// Result of a linkage operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageResult {
    pub status: LinkageStatus,
    pub client_data: Option<ClientLinkageData>,
    pub server_data: Option<ServerLinkageData>,
    pub linkage_id: LinkageID,
}

impl LinkageResult {
    pub fn new(
        status: LinkageStatus,
        client_data: Option<ClientLinkageData>,
        server_data: Option<ServerLinkageData>,
        linkage_id: LinkageID,
    ) -> Self {
        Self {
            status,
            client_data,
            server_data,
            linkage_id,
        }
    }

    pub fn pending(linkage_id: LinkageID) -> Self {
        Self {
            status: LinkageStatus::Pending,
            client_data: None,
            server_data: None,
            linkage_id,
        }
    }

    pub fn complete(
        client_data: ClientLinkageData,
        server_data: ServerLinkageData,
        linkage_id: LinkageID,
    ) -> Self {
        Self {
            status: LinkageStatus::Complete,
            client_data: Some(client_data),
            server_data: Some(server_data),
            linkage_id,
        }
    }

    pub fn failed(linkage_id: LinkageID) -> Self {
        Self {
            status: LinkageStatus::Failed,
            client_data: None,
            server_data: None,
            linkage_id,
        }
    }
}
