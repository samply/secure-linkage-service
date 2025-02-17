use std::rc::Rc;

/// Simple types (wrappers)

#[derive(Debug, Clone, Copy)]
pub struct Counter(pub u64);

impl Counter {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ready(pub bool);

impl Ready {
    pub fn new(value: bool) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Match(pub bool);

impl Match {
    pub fn new(value: bool) -> Self {
        Self(value)
    }
}

/// String-containing types using Rc<str>

#[derive(Debug, Clone)]
pub struct HostId(pub Rc<str>);

impl HostId {
    pub fn new(id: &str) -> Self {
        Self(Rc::from(id))
    }
}

#[derive(Debug, Clone)]
pub struct Key(pub Rc<str>);

impl Key {
    pub fn new(value: &str) -> Self {
        Self(Rc::from(value))
    }
}

#[derive(Debug, Clone)]
pub struct Padding(pub Rc<str>);

impl Padding {
    pub fn new(value: &str) -> Self {
        Self(Rc::from(value))
    }
}

#[derive(Debug, Clone)]
pub struct Index(pub Rc<str>);

impl Index {
    pub fn new(value: &str) -> Self {
        Self(Rc::from(value))
    }
}

/// Composite types

#[derive(Debug, Clone)]
pub struct EpiLinker {
    pub uid: HostId,
    pub key: Key,
    pub padding: Padding,
    pub id_counter: Option<Counter>,
}

impl EpiLinker {
    pub fn new(uid: HostId, key: Key, padding: Padding, id_counter: Option<Counter>) -> Self {
        Self {
            uid,
            key,
            padding,
            id_counter,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinkageShare {
    pub matches: Rc<[Match]>,
    pub tentative_matches: Rc<[Match]>,
    pub indexes: Option<Rc<[Index]>>,
}

impl LinkageShare {
    pub fn new(matches: &[Match], tentative_matches: &[Match], indexes: Option<&[Index]>) -> Self {
        Self {
            matches: Rc::from(matches),
            tentative_matches: Rc::from(tentative_matches),
            indexes: indexes.map(Rc::from),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HostRole {
    Client,
    Server,
}

#[derive(Debug, Clone)]
pub struct Computation {
    pub local_id: HostId,
    pub remote_id: HostId,
    pub role: HostRole,
    pub data: Option<LinkageShare>,
}

impl Computation {
    pub fn new(
        local_id: HostId,
        remote_id: HostId,
        role: HostRole,
        data: Option<LinkageShare>,
    ) -> Self {
        Self {
            local_id,
            remote_id,
            role,
            data,
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct LinkageResult {
    pub state: LinkageState,
    pub local_id: HostId,
    pub remote_id: HostId,
}

impl LinkageResult {
    pub fn new(state: LinkageState, local_id: HostId, remote_id: HostId) -> Self {
        Self {
            state,
            local_id,
            remote_id,
        }
    }
}
