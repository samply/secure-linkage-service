use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

/// Simple types (wrappers), and their enums

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeOut(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TentativeMatch(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmedMatch(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchBit {
    ConfirmedMatch,
    TentativeMatch,
}

pub enum MatchResult {
    Match,
    NoMatch,
}

/// String-containing types using Rc<str>

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostId(pub Rc<str>);

impl HostId {
    pub fn new(value: &str) -> Self {
        Self(Rc::from(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key(pub Rc<str>);

impl Key {
    pub fn new(value: &str) -> Self {
        Self(Rc::from(value))
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct RecordId(pub Rc<str>);

impl RecordId {
    pub fn new(value: &str) -> Self {
        Self(Rc::from(value))
    }
}

impl Deref for RecordId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Composite types, and their enums

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiLinker {
    pub uid: HostId,
    pub key: Key,
    pub padding: Padding,
    pub linkage_data: Option<LinkageData>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageShare {
    tmb: TentativeMatchBit,
    cmd: ConfirmedMatchBit,
    bmi: BestMatchIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkageData {
    ClientLinkageData,
    ServerLinkageData,
}

#[derive(Debug, Clone)]
pub struct ClientLinkageData {
    pub share: Rc<Vec<LinkageShare>>,
}

#[derive(Debug, Clone)]
pub struct ServerLinkageData {
    pub share: Rc<Vec<LinkageShare>>,
    pub ids: Rc<Vec<RecordId>>,
}

impl ServerLinkageData {
    pub fn new(share: &LinkageShare, ids: &[RecordId]) -> Self {
        Self {
            share: Rc::from(share.clone()),
            ids: Rc::from(ids.to_vec()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Computation {
    pub client_id: HostId,
    pub server_id: HostId,
    pub client_data: Option<ClientLinkageData>,
    pub_server_data: Option<ServerLinkageData>,

 // You will update the computation when a new set of data comes in
}


update_computation(computation, server_data, client_data) --> linkage_result

impl Computation {
    pub fn new(client_id: HostId, server_id: HostId, data: Option<LinkageShare>) -> Self {
        Self {
            client_id,
            server_id,
            data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageState {
    pub epilinkers: Rc<[EpiLinker]>,
    pub computations: Rc<[Computation]>,
}

impl LinkageState {
    pub fn new(epilinkers: &[EpiLinker], computations: &[Computation]) -> Self {
        Self {
            epilinkers: Rc::from(epilinkers),
            computations: Rc::from(computations),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkageResult {
    pub state: LinkageState,
    pub client_id: HostId,
    pub server_id: HostId,
}

impl LinkageResult {
    pub fn new(state: LinkageState, client_id: HostId, server_id: HostId) -> Self {
        Self {
            state,
            client_id,
            server_id,
        }
    }
}
