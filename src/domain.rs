use base64::prelude::BASE64_STANDARD;
use base64::{DecodeError, Engine};
use std::fmt;
use std::num::ParseIntError;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchBit {
    Unset,
    Set,
}

impl MatchBit {
    pub fn to_u64(&self) -> u64 {
        match self {
            MatchBit::Unset => 0,
            MatchBit::Set => 1,
        }
    }

    pub fn from_u64(value: u64) -> Self {
        match value {
            0 => MatchBit::Unset,
            _ => MatchBit::Set,
        }
    }

    pub fn xor(&self, other: Self) -> Self {
        MatchBit::from_u64(self.to_u64() ^ other.to_u64())
    }
}

//
// Simple wrapper types for match data
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TentativeMatchBit(pub MatchBit);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmedMatchBit(pub MatchBit);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexOfBestMatch(pub u64);

//
// String-containing types using Rc<str> for efficient sharing
//

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Base64EncodedString(pub Rc<str>);

impl Base64EncodedString {
    pub fn new(value: &str) -> Result<Self, LinkageError> {
        BASE64_STANDARD.decode(value)?;
        Ok(Self(Rc::from(value)))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordId(pub Base64EncodedString);

impl RecordId {
    pub fn new(value: &str) -> Result<Self, LinkageError> {
        Ok(Self(Base64EncodedString::new(value)?))
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

impl ResultData {
    pub fn has_a_match(&self) -> bool {
        self.confirmed_match.0 == MatchBit::Set || self.tentative_match.0 == MatchBit::Set
    }
}

// TODO: maybe delete this `new` function?
impl ResultData {
    pub fn new(
        confirmed_match: ConfirmedMatchBit,
        tentative_match: TentativeMatchBit,
        index_of_best_match: IndexOfBestMatch,
    ) -> Self {
        Self {
            confirmed_match,
            tentative_match,
            index_of_best_match,
        }
    }

    pub fn from_strings(
        tentative_match: &str,
        confirmed_match: &str,
        index: &str,
    ) -> Result<Self, LinkageError> {
        Ok(Self {
            confirmed_match: ConfirmedMatchBit(MatchBit::from_u64(confirmed_match.parse()?)),
            tentative_match: TentativeMatchBit(MatchBit::from_u64(tentative_match.parse()?)),
            index_of_best_match: IndexOfBestMatch(index.parse()?),
        })
    }

    // Default ResultData with all zeros
    pub fn empty() -> Self {
        Self {
            confirmed_match: ConfirmedMatchBit(MatchBit::Unset),
            tentative_match: TentativeMatchBit(MatchBit::Unset),
            index_of_best_match: IndexOfBestMatch(0),
        }
    }
}

impl fmt::Display for ResultData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "match-{:?}-{:?}-{}",
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
        self.record_ids.get(index)
    }
}

//
// Linkage ID and related types
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageID(pub Base64EncodedString);

impl LinkageID {
    pub fn new(value: &str) -> Result<Self, LinkageError> {
        Ok(Self(Base64EncodedString::new(value)?))
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

//
// Error handling
//

#[derive(Debug, thiserror::Error)]
pub enum LinkageError {
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Base64 decoding error: {0}")]
    Base64DecodingError(#[from] DecodeError),
    #[error("No input data for linkage")]
    EmptyInput,
    #[error("Invalid number format: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Cannot create random key material")]
    CannotCreateRandomKeyMaterial,
    #[error("Cannot create LessSafeKey")]
    CannotCreateLessSafeKey,
    #[error("Invalid nonce size")]
    InvalidLinkageId,
    #[error("Invalid Linkage Id")]
    InvalidNonceSize,
    #[error("Cannot not generate Nonce")]
    CannotGenerateNonce,
    #[error("Cannot encrypt payload")]
    CannotEncryptPayload,
    #[error("Decoded ID too short")]
    DecodedIdTooShort,
    #[error("ServerID not found for best match index")]
    ServerIDNotFoundForBestMatchIndex,
    #[error("Client and server data have different lengths")]
    MismatchedDataLength,
}
