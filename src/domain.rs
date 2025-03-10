use std::rc::Rc;

//
// Simple wrapper types that don't need Rc
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TentativeMatchBit(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmedMatchBit(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BestMatchIndex(pub u64);

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
// Shared data structures
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareData {
    pub tentative_match: TentativeMatchBit,
    pub confirmed_match: ConfirmedMatchBit,
    pub index_of_best_match: BestMatchIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerShareData {
    pub share_data: ShareData,
    pub id: RecordId,
}

//
// Linkage data structures
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLinkageData {
    pub share: Rc<Vec<ShareData>>,
}

impl ClientLinkageData {
    pub fn new(share_data: Vec<ShareData>) -> Self {
        Self {
            share: Rc::new(share_data),
        }
    }

    pub fn empty() -> Self {
        Self {
            share: Rc::new(Vec::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLinkageData {
    pub share_data: Rc<Vec<ServerShareData>>,
}

impl ServerLinkageData {
    pub fn new(share_data: Vec<ShareData>, ids: Vec<RecordId>) -> Self {
        // Ensure share_data and ids have the same length
        assert_eq!(
            share_data.len(),
            ids.len(),
            "Share data and IDs must have the same length"
        );

        // Zip the share_data and ids together
        let server_share = share_data
            .into_iter()
            .zip(ids.into_iter())
            .map(|(share, id)| ServerShareData {
                share_data: share,
                id,
            })
            .collect();

        Self {
            share_data: Rc::new(server_share),
        }
    }

    pub fn empty() -> Self {
        Self {
            share_data: Rc::new(Vec::new()),
        }
    }

    // Helper to extract just the IDs
    pub fn ids(&self) -> Vec<RecordId> {
        self.share_data
            .iter()
            .map(|server_share| server_share.id.clone())
            .collect()
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
