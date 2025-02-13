struct HostId(str);
struct Key(str);
struct Padding(str);
struct LocalId(UId);
struct RemoteId(UId);
struct Match(bool);
struct Index(str);
struct Counter(u64);
struct Ready(bool);

struct EpiLinker {
    uid: HostId,
    key: Key,
    padding: Padding,
    id_counter: Option<Counter>,
}

struct LinkageShare {
    matches: Vec<Match>,
    tentative_matches: Vec<Match>,
    indexes: Optional<Vec<Index>>,
}

enum HostRole {
    Client,
    Server,
}

struct Computation {
    local_id: HostId,
    remote_id: HostId,
    role: HostRole,
    data: Optional<LinkageShare>,
}

struct LinkageState {
    epilinkers: Vec<EpiLinker>,
    computations: Vec<Computation>,
}

struct LinkageResult {
    state: LinkageState,
    local_id: HostId,
    remote_id: HostId,
}
