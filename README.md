# Secure Linkage Service

[![Rust](https://img.shields.io/badge/rust-1.85.1+-orange.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Secure Linkage Service is a Rust web service for multi-party privacy-preserving record linkage[^1]. It lets two organizations coordinate linkage results without sending each other plaintext identifiers.

The service exposes HTTP endpoints for health checks, fresh encrypted record IDs, and paired linkage-result submission. It uses ChaCha20-Poly1305 for authenticated encryption and XOR-based match-bit combination for the linkage step.

## Features

- ChaCha20-Poly1305 AEAD encryption with random nonces
- Paired client/server linkage requests between named parties
- Fresh encrypted record ID generation
- Async HTTP server built on Axum and Tokio
- Health endpoint, structured tracing, and input validation
- Unit, integration, and end-to-end test coverage

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    REST API Layer                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │   Health    │ │Fresh Record │ │      Linkage        │    │
│  │ Endpoints   │ │     IDs     │ │    Endpoints        │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Services Layer                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │   Party     │ │   Linkage   │ │  Fresh Record IDs   │    │
│  │  Service    │ │   Service   │ │      Service        │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                     Core Layer                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │ Cryptography│ │   Linkage   │ │       Domain        │    │
│  │   Module    │ │   Logic     │ │       Models        │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Quick start

### Prerequisites

- Rust 1.89 or newer
- Git
- [just](https://github.com/casey/just)

### Build and test

```bash
git clone https://github.com/your-org/secure-linkage-service.git
cd secure-linkage-service
cargo build --release
cargo test
```

### Run the server

```bash
just start
```

`just start` uses `linkage-parties.toml`. Pass an empty config argument to use the built-in test parties:

```bash
just start ""
```

Stop it with:

```bash
just stop
```

The server listens on `http://localhost:5000` by default.

### Check health

```bash
curl http://localhost:5000/health
```

Expected response:

```json
{
  "status": "healthy",
  "service": "linkage-service",
  "timestamp": "2025-01-30T10:30:00Z",
  "version": "0.1.0"
}
```

## API

Base URL:

```text
http://localhost:5000
```

### Health check

```http
GET /health
```

Returns service health and version metadata.

### Generate fresh record IDs

```http
POST /freshRecordIDs/{party}?n={count}
```

Parameters:

- `party`: party name, such as `dkfz` or `tuda`
- `n`: number of IDs to generate, defaults to `1`, maximum `1000`

Example:

```bash
curl -X POST "http://localhost:5000/freshRecordIDs/dkfz?n=5"
```

Response:

```json
{
  "status": "success",
  "party": "dkfz",
  "count": 5,
  "fresh_record_ids": [
    "Y3J5cHRvZ3JhcGhpY2FsbHlfZW5jcnlwdGVkX2RhdGE=",
    "c2VjdXJlX2xpbmthZ2VfZGF0YV93aXRoX3Byb3Blcg=="
  ],
  "message": null
}
```

### Submit linkage results

```http
POST /linkageResult/{source_party}/{destination_party}
```

The service pairs a `server` request with the matching `client` request for the opposite party direction.

Server request:

```json
{
  "role": "server",
  "result": [
    {
      "match": 0,
      "tentativeMatch": 1,
      "bestIndex": 0
    }
  ],
  "ids": ["base64_encoded_encrypted_id"]
}
```

Client request:

```json
{
  "role": "client",
  "result": [
    {
      "match": 1,
      "tentativeMatch": 0,
      "bestIndex": 0
    }
  ]
}
```

Example workflow:

```bash
curl -X POST "http://localhost:5000/linkageResult/dkfz/tuda" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "server",
    "result": [{"match": 0, "tentativeMatch": 1, "bestIndex": 0}],
    "ids": ["Y3J5cHRvZ3JhcGhpY2FsbHlfZW5jcnlwdGVkX2RhdGE="]
  }' &

curl -X POST "http://localhost:5000/linkageResult/tuda/dkfz" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "client",
    "result": [{"match": 1, "tentativeMatch": 0, "bestIndex": 0}]
  }'
```

Response:

```json
{
  "status": "completed",
  "linkage_ids": [
    "ZW5jcnlwdGVkX2xpbmthZ2VfaWRfcmVzdWx0XzE="
  ],
  "message": "Linkage computation completed"
}
```

## Cryptographic protocol

For each linkage round:

1. Each party uses a 256-bit ChaCha20 key.
2. The server-side party sends encrypted record IDs.
3. The client and server submit match bits.
4. The service XORs the submitted bits.
5. For a match, the service derives the linkage ID from the server ID.
6. For a non-match, the service generates a random linkage ID.

Match-bit table:

```text
Client Bits    Server Bits    XOR Result    Outcome
match=1        match=0        1             MATCH FOUND
tentative=0    tentative=1    1             MATCH FOUND
match=0        match=0        0             NO MATCH
tentative=0    tentative=0    0             NO MATCH
```

Security properties:

- Parties do not receive each other's plaintext identifiers.
- ChaCha20-Poly1305 detects ciphertext tampering.
- Random nonces protect repeated encryption operations.
- Rust memory safety prevents buffer overflows and use-after-free bugs in safe code.

## Testing

Run unit and integration tests:

```bash
cargo test
```

Run the end-to-end suite:

```bash
just e2e
```

Run a single end-to-end scenario:

```bash
./tests/e2e/run-single-test.sh 01  # basic client-server flow
./tests/e2e/run-single-test.sh 05  # multiple matches with real crypto
./tests/e2e/run-single-test.sh 14  # performance test
```

The test suite covers fresh ID generation, paired request coordination, concurrent requests, large inputs, malformed requests, and error paths.

## Performance

Local test runs exercise these workloads:

| Metric | Current test coverage |
| --- | --- |
| Concurrent requests | 100 or more |
| Typical request latency | Under 1 second for tested datasets |
| Dataset size | 1000 or more records |
| Scaling behavior | Linear in tested cases |

Example checks:

```bash
time curl -X POST "http://localhost:5000/freshRecordIDs/dkfz?n=1000"
```

```bash
# Submit paired linkage requests with 100 records to measure linkage latency.
```

## Deployment

### Development

```bash
just start
just stop
```

### Docker

```dockerfile
FROM rust:1.85-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/secure-linkage-service /usr/local/bin/
EXPOSE 5000
CMD ["secure-linkage-service"]
```

```bash
docker build -t secure-linkage-service .
docker run -p 5000:5000 secure-linkage-service
```

Production deployments should add TLS termination, request limits, log collection, and a non-root container user.

Environment variables:

```bash
HOST=127.0.0.1                              # bind address
PORT=5000                                   # server port
PARTIES_CONFIG_FILE=./linkage-parties.toml  # known linkage parties
RUST_LOG=info                               # tracing level
RUST_BACKTRACE=1                            # backtraces during development
```

## Development

Project layout:

```text
secure-linkage-service/
├── src/
│   ├── core/
│   │   ├── crypto.rs
│   │   └── linkage.rs
│   ├── domain.rs
│   ├── services/
│   │   ├── party_service.rs
│   │   ├── linkage_service.rs
│   │   └── fresh_record_ids_service.rs
│   ├── rest_api/
│   │   ├── health/
│   │   ├── linkage/
│   │   ├── fresh_record_ids/
│   │   └── party/
│   ├── server.rs
│   └── main.rs
├── tests/
│   ├── e2e/
│   └── *.rs
├── Cargo.toml
└── README.md
```

Common commands:

```bash
just fmt
just lint
just test
just e2e
```

Add new code in the layer that owns the behavior:

- `src/core/` for cryptography and linkage algorithms
- `src/services/` for request coordination and application rules
- `src/rest_api/` for HTTP handlers and routing
- `tests/` for unit, integration, and end-to-end coverage

## Security

The service addresses these risks in code:

- Plaintext identifier exposure between parties
- Ciphertext tampering
- Replay through nonce reuse
- Malformed input
- Memory corruption in safe Rust code

Deployers still need to provide transport security, authentication, authorization, rate limits, and operational monitoring.

Report security vulnerabilities by email to [Tobias Kussel](mailto:tobias.kussel@dkfz-heidelberg.de). Do not file public issues for security reports.

## Contributing

Pull requests are welcome.

1. Fork the repository.
2. Create a feature branch: `git checkout -b feature/my-change`.
3. Add code and tests.
4. Run `just test && just e2e`.
5. Commit your changes.
6. Push your branch.
7. Open a pull request.

This project follows the [Free Software Camp Code of Conduct](https://camp.fsf.org.in/code-of-conduct).

## License

This program is free software. You can redistribute it and modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 or any later version.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <http://www.gnu.org/licenses/>.

AGPL points for service deployers:

- You may use, study, modify, and distribute the software.
- You must disclose source code for modifications.
- Network use triggers the source-distribution requirement.
- Derivative works must use a compatible open-source license.

See [LICENSE](LICENSE) for the full license text.

Copyright:

- Rust reimplementation: Copyright © 2025 Leandro Doctors(*)
- Python core functional overhaul: Copyright © 2025 Leandro Doctors(*)
- Original Python imperative implementation: Copyright © 2018 Tobias Kussel(*)

_(*) affiliated with the [Federated Information Systems Department](https://www.dkfz.de/en/federated-information-systems), German Cancer Research Center (DKFZ) / University of Mannheim (UMM), Germany_

## Acknowledgments

Leandro Doctors thanks Tobias Kussel for problem-domain guidance, the initial Rust AEAD implementation, and review.

## Reference

[^1]: Stammler S, Kussel T, Schoppmann P, Stampe F, Tremper G, Katzenbeisser S, Hamacher K, Lablans M. Mainzelliste SecureEpiLinker (MainSEL): privacy-preserving record linkage using secure multi-party computation. *Bioinformatics*. 2022 Mar 4;38(6):1657-1668. doi: [10.1093/bioinformatics/btaa764](https://doi.org/10.1093/bioinformatics/btaa764). PMID: 32871006; PMCID: PMC8896632.
