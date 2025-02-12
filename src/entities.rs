struct EpiLinker {
    uid: str,
    key: str,
    padding: str,
    id_counter: Option<int64>,
}

struct LinkageShare {
    matches: Vec<bool>,
    tentative_matches: Vec<bool>,
    indexes: Optional<Vec<str>>,
}

struct Computation {
    local_id: str,
    remote_id: str,
    role: str,
    data: Optional<LinkageShare>,
}

struct LinkageState {
    epilinkers: Vec<EpiLinker>,
    computations: Vec<Computation>,
}
