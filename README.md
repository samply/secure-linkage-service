# Secure Linkage Service

[![Rust](https://img.shields.io/badge/rust-1.85.1+-orange.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

A high-performance, cryptographically secure web service for multy-party privacy-preserving record linkage[^1].
This service enables organizations to identify matching records without revealing sensitive data content.

## 🌟 Features

- **🔐 Cryptographically Secure**: ChaCha20-Poly1305 AEAD encryption with secure random nonce generation
- **🤝 Multi-Party Privacy**: XOR-based secure computation ensures parties never see each other's raw data
- **🚀 High Performance**: Async/await architecture handling 100+ concurrent requests with sub-second latency
- **🛡️ Production Ready**: Comprehensive error handling, input validation, and security measures
- **📊 Scalable**: Tested with datasets up to 1000+ records with linear scaling
- **🔍 Observable**: Built-in health checks, structured logging, and comprehensive test coverage

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    REST API Layer                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │   Health    │ │Fresh Record │ │      Linkage        │    │
│  │ Endpoints   │ │     IDs     │ │    Endpoints        │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Services Layer                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │   Party     │ │   Linkage   │ │  Fresh Record IDs   │    │
│  │  Service    │ │   Service   │ │      Service        │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                     Core Layer                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │ Cryptography│ │   Linkage   │ │       Domain        │    │
│  │   Module    │ │   Logic     │ │       Models        │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- **Rust 1.85.1+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/secure-linkage-service.git
cd secure-linkage-service

# Build the project
cargo build --release

# Run tests to verify installation
cargo test
```

### Running the Service

```bash
# Start the server (development mode)
cargo run

# Or use the provided script
./start-server.sh

# The service will be available at http://localhost:5000
```

### Quick Health Check

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

## 📚 API Documentation

### Base URL
```
http://localhost:5000
```

### Endpoints

#### Health Check
```http
GET /health
```

Returns system health and version information.

**Response:**
```json
{
  "status": "healthy",
  "service": "linkage-service",
  "timestamp": "2025-01-30T10:30:00Z",
  "version": "0.1.0"
}
```

#### Generate Fresh Record IDs

```http
POST /freshRecordIDs/{party}?n={count}
```

Generates cryptographically secure record identifiers.

**Parameters:**
- `party` (path) - Party name (e.g., "dkfz", "tuda")
- `n` (query, optional) - Number of IDs to generate (default: 1, max: 1000)

**Example:**
```bash
curl -X POST "http://localhost:5000/freshRecordIDs/dkfz?n=5"
```

**Response:**
```json
{
  "status": "success",
  "party": "dkfz",
  "count": 5,
  "fresh_record_ids": [
    "Y3J5cHRvZ3JhcGhpY2FsbHlfZW5jcnlwdGVkX2RhdGE=",
    "c2VjdXJlX2xpbmthZ2VfZGF0YV93aXRoX3Byb3Blcg==",
    "..."
  ],
  "message": null
}
```

#### Secure Record Linkage

```http
POST /linkageResult/{source_party}/{destination_party}
```

Performs secure record linkage between two parties.

**Server Request Format:**
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

**Client Request Format:**
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

**Example Workflow:**
```bash
# Server request (runs in background, waits for matching client)
curl -X POST "http://localhost:5000/linkageResult/dkfz/tuda" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "server",
    "result": [{"match": 0, "tentativeMatch": 1, "bestIndex": 0}],
    "ids": ["Y3J5cHRvZ3JhcGhpY2FsbHlfZW5jcnlwdGVkX2RhdGE="]
  }' &

# Client request (pairs with server request)
curl -X POST "http://localhost:5000/linkageResult/tuda/dkfz" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "client",
    "result": [{"match": 1, "tentativeMatch": 0, "bestIndex": 0}]
  }'
```

**Response:**
```json
{
  "status": "completed",
  "linkage_ids": [
    "ZW5jcnlwdGVkX2xpbmthZ2VfaWRfcmVzdWx0XzE="
  ],
  "message": "Linkage computation completed"
}
```

## 🔐 Cryptographic Protocol

### How It Works

1. **Key Generation**: Each party generates a unique 256-bit ChaCha20 key
2. **Data Encryption**: Records encrypted with ChaCha20-Poly1305 AEAD + random nonces
3. **Secure Matching**: Parties share encrypted match bits, XOR determines matches
4. **Linkage Generation**:
   - **Matches**: Decrypt server ID → create new encrypted linkage ID
   - **Non-matches**: Generate cryptographically random linkage ID

### Security Properties

- **Zero-Knowledge**: Parties never see each other's plaintext data
- **Authenticated Encryption**: ChaCha20-Poly1305 prevents tampering
- **Replay Protection**: Unique nonces prevent replay attacks
- **Perfect Forward Secrecy**: Session keys are ephemeral

### Match Bit Logic

The XOR-based matching works as follows:

```
Client Bits    Server Bits    XOR Result    Outcome
match=1        match=0        1            → MATCH FOUND
tentative=0    tentative=1    1            → MATCH FOUND
match=0        match=0        0            → NO MATCH
tentative=0    tentative=0    0            → NO MATCH
```

## 🧪 Testing

### Run All Tests
```bash
# Unit and integration tests
cargo test

# End-to-end tests (requires running server)
./run-e2e-tests.sh
```

### Test Categories

#### Unit Tests
- Cryptographic operations (encryption/decryption)
- Domain logic (XOR operations, match bit handling)
- Service layer business logic

#### Integration Tests
- REST API endpoints
- Multi-party request coordination
- Error handling and validation

#### End-to-End Tests
18+ comprehensive scenarios covering:

```bash
# Run specific test scenarios
./tests/e2e/run-single-test.sh 01  # Basic client-server flow
./tests/e2e/run-single-test.sh 05  # Multiple matches with real crypto
./tests/e2e/run-single-test.sh 14  # Performance testing
```

**Test Scenarios:**
- ✅ Fresh Record ID generation
- ✅ Basic client-server coordination
- ✅ Concurrent request handling
- ✅ Large dataset processing (1000+ records)
- ✅ Security validation (invalid parties, malformed data)
- ✅ Performance and stress testing
- ✅ Edge cases and error conditions

## 📊 Performance

### Benchmarks

| Metric | Performance |
|--------|-------------|
| **Throughput** | 100+ concurrent requests |
| **Latency** | < 1 second for typical datasets |
| **Memory Usage** | Efficient streaming, minimal footprint |
| **Scalability** | Linear scaling with dataset size |
| **Max Dataset** | 1000+ records tested |

### Load Testing Results

```bash
# Generate 1000 fresh IDs
time curl -X POST "http://localhost:5000/freshRecordIDs/dkfz?n=1000"
# Typical result: ~200-500ms

# Process 100 record linkage
# Typical result: ~300-800ms depending on matches
```

## 🐳 Deployment

### Development
```bash
./start-server.sh
```

### Production with Docker

```dockerfile
FROM rust:1.85-slim as builder
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
# Build and run
docker build -t secure-linkage-service .
docker run -p 5000:5000 secure-linkage-service
```

### Production Considerations

- **Reverse Proxy**: Use nginx/traefik for TLS termination
- **Rate Limiting**: Implement request rate limiting
- **Monitoring**: Deploy with logging aggregation (ELK, Prometheus)
- **Scaling**: Service is stateless, can be horizontally scaled
- **Security**: Run with non-root user, minimal container image

### Environment Variables

```bash
PORT=5000                    # Server port (default: 5000)
RUST_LOG=info               # Log level
RUST_BACKTRACE=1            # Error backtraces (development)
```

## 🛠️ Development

### Project Structure

```
secure-linkage-service/
├── src/
│   ├── core/                 # Core cryptographic and linkage logic
│   │   ├── crypto.rs        # ChaCha20-Poly1305 encryption
│   │   └── linkage.rs       # Secure linkage algorithms
│   ├── domain.rs            # Domain models and types
│   ├── services/            # Business logic services
│   │   ├── party_service.rs
│   │   ├── linkage_service.rs
│   │   └── fresh_record_ids_service.rs
│   ├── rest_api/            # HTTP API layer
│   │   ├── health/
│   │   ├── linkage/
│   │   ├── fresh_record_ids/
│   │   └── party/
│   ├── server.rs            # Server configuration
│   └── main.rs              # Application entry point
├── tests/                   # Test suites
│   ├── e2e/                 # End-to-end tests
│   └── *.rs                 # Unit/integration tests
├── Cargo.toml              # Dependencies and metadata
└── README.md               # This file
```

### Key Dependencies

```toml
[dependencies]
axum = "0.8"           # Modern async web framework
tokio = "1.47"         # Async runtime
ring = "0.17"          # Cryptographic operations
serde = "1.0"          # Serialization
base64 = "0.22"        # Binary encoding
tracing = "0.1"        # Structured logging
uuid = "1.17"          # UUID generation
chrono = "0.4"         # Date/time handling
```

### Adding New Features

1. **Core Logic**: Add to `src/core/` for cryptographic or algorithmic changes
2. **Business Logic**: Add to `src/services/` for new business capabilities
3. **API Endpoints**: Add to `src/rest_api/` for new HTTP endpoints
4. **Tests**: Add corresponding tests in `tests/` directory

### Code Style

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for security issues
cargo audit
```

## 🔒 Security

### Threat Model

**Protected Against:**
- ✅ Data exposure between parties
- ✅ Replay attacks (unique nonces)
- ✅ Tampering (authenticated encryption)
- ✅ Timing attacks (consistent processing)
- ✅ Memory safety issues (Rust ownership)
- ✅ Input validation attacks

**Security Measures:**
- **Cryptographic**: ChaCha20-Poly1305 AEAD with secure random nonces
- **Network**: HTTPS recommended for production deployment
- **Input Validation**: All inputs sanitized and validated
- **Memory Safety**: Rust prevents buffer overflows and use-after-free
- **Access Control**: Party-based authentication and authorization

### Reporting Security Issues

Please report security vulnerabilities via email to [Tobias Kussel](tobias.kussel@dkfz-heidelberg.de).
Do not file public issues for security vulnerabilities.

## 🤝 Contributing

We welcome pull requests!

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes with tests
4. Run the test suite: `cargo test && ./run-e2e-tests.sh`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

### Code of Conduct

This project follows the [Free Software Camp Code of Conduct](https://camp.fsf.org.in/code-of-conduct)

## 📄 License

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see http://www.gnu.org/licenses/.
**Key Points:**
- ✅ Free to use, analyze, modify, and distribute
- ✅ Must disclose source code of any modifications
- ✅ Network use triggers copyleft (unlike GPL)
- ✅ Ensures derivative works remain open source

See the [LICENSE](LICENSE) file for full details.

**Copyright Notice:**
- This Rust re-implementation: Copyright © 2025 Leandro Doctors(*)
- Python core functional overhaul: Copyright © 2025 Leandro Doctors(*)
- Original Python imperative implementation: Copyright © 2018 Tobias Kussel(*)

_(*) currently affiliated with the [Federated Information Systems Department](https://www.dkfz.de/en/federated-information-systems), German Cancer Research Center (DKFZ) / University of Mannheim (UMM), Germany_

## 🙏 Acknowledgments

Leandro Doctors thanks Tobias Kussel for the insight into the problem domain, the initial AEAD implementation in Rust, and for reviewing his own.

**Reference:**

[^1]: Stammler S, Kussel T, Schoppmann P, Stampe F, Tremper G, Katzenbeisser S, Hamacher K, Lablans M. Mainzelliste SecureEpiLinker (MainSEL): privacy-preserving record linkage using secure multi-party computation. *Bioinformatics*. 2022 Mar 4;38(6):1657-1668. doi: [10.1093/bioinformatics/btaa764](https://doi.org/10.1093/bioinformatics/btaa764). PMID: 32871006; PMCID: PMC8896632.
---

**Built with ❤️ for privacy-preserving data sharing.**
