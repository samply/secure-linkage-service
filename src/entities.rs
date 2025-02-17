use std::rc::Rc;

/// Simple types (wrappers)

#[derive(Debug, Clone, Copy)]
pub struct Counter(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct Ready(pub bool);

#[derive(Debug, Clone, Copy)]
pub struct Match(pub bool);

/// String-containing types using Rc<str>

#[derive(Debug, Clone)]
pub struct HostId(pub Rc<str>);

#[derive(Debug, Clone)]
pub struct Key(pub Rc<str>);

#[derive(Debug, Clone)]
pub struct Padding(pub Rc<str>);

#[derive(Debug, Clone)]
pub struct Index(pub Rc<str>);

/// Composite types

#[derive(Debug, Clone)]
pub struct EpiLinker {
    pub uid: HostId,
    pub key: Key,
    pub padding: Padding,
    pub id_counter: Option<Counter>,
}

#[derive(Debug, Clone)]
pub struct LinkageShare {
    pub matches: Rc<[Match]>,
    pub tentative_matches: Rc<[Match]>,
    pub indexes: Option<Rc<[Index]>>,
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

#[derive(Debug, Clone)]
pub struct LinkageState {
    pub epilinkers: Rc<[EpiLinker]>,
    pub computations: Rc<[Computation]>,
}

#[derive(Debug, Clone)]
pub struct LinkageResult {
    pub state: LinkageState,
    pub local_id: HostId,
    pub remote_id: HostId,
}
