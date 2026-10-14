set shell := ["bash", "-cu"]

binary := "secure-linkage-service"
host := "localhost"
port := "5000"
pidfile := ".server.pid"
logfile := "server.log"
config_file := "linkage-parties.toml"
rust_image := "rust:1.89-bookworm"

_default:
    @just --list

build:
    cargo build --release

start config=config_file:
    #!/usr/bin/env bash
    set -euo pipefail

    binary="{{ binary }}"
    host="{{ host }}"
    port="{{ port }}"
    pidfile="{{ pidfile }}"
    logfile="{{ logfile }}"
    config="{{ config }}"

    if curl -fsS --max-time 1 "http://${host}:${port}/health" >/dev/null 2>&1; then
      echo "Server is already responding at http://${host}:${port}"
      exit 0
    fi

    if [[ -f "$pidfile" ]]; then
      old_pid="$(cat "$pidfile")"
      if [[ -n "$old_pid" ]] && kill -0 "$old_pid" 2>/dev/null; then
        echo "PID file exists and process $old_pid is running, but health check failed."
        echo "Run: just stop"
        exit 1
      fi
      rm -f "$pidfile"
    fi

    cargo build --release

    if [[ -n "$config" ]]; then
      if [[ ! -f "$config" ]]; then
        echo "Parties config file not found: $config" >&2
        exit 1
      fi
      export PARTIES_CONFIG_FILE="$PWD/$config"
      echo "Using parties config: $PARTIES_CONFIG_FILE"
    fi

    "target/release/${binary}" > "$logfile" 2>&1 &
    pid="$!"
    echo "$pid" > "$pidfile"

    echo "Started ${binary} as PID ${pid}"
    echo "Logs: ${logfile}"

    for _ in $(seq 1 60); do
      if curl -fsS --max-time 1 "http://${host}:${port}/health" >/dev/null 2>&1; then
        echo "Server is ready at http://${host}:${port}"
        exit 0
      fi

      if ! kill -0 "$pid" 2>/dev/null; then
        echo "Server exited during startup. Logs:"
        cat "$logfile" 2>/dev/null || true
        rm -f "$pidfile"
        exit 1
      fi

      sleep 1
    done

    echo "Server did not become ready within 60 seconds. Logs:"
    cat "$logfile" 2>/dev/null || true
    kill "$pid" 2>/dev/null || true
    rm -f "$pidfile"
    exit 1

stop:
    #!/usr/bin/env bash
    set -euo pipefail

    pidfile="{{ pidfile }}"
    binary="{{ binary }}"

    if [[ -f "$pidfile" ]]; then
      pid="$(cat "$pidfile")"
      if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
        echo "Stopping ${binary} PID ${pid}"
        kill "$pid" 2>/dev/null || true

        for _ in $(seq 1 10); do
          if ! kill -0 "$pid" 2>/dev/null; then
            rm -f "$pidfile"
            echo "Stopped"
            exit 0
          fi
          sleep 1
        done

        echo "Process did not exit after SIGTERM; sending SIGKILL"
        kill -9 "$pid" 2>/dev/null || true
      fi
      rm -f "$pidfile"
      echo "Stopped"
      exit 0
    fi

    pids="$(pgrep -f "target/release/${binary}|target/debug/${binary}|(^|/)${binary}$" 2>/dev/null || true)"
    if [[ -z "$pids" ]]; then
      echo "No ${binary} process found"
      exit 0
    fi

    echo "Stopping ${binary} process(es): ${pids}"
    kill $pids 2>/dev/null || true

e2e:
    #!/usr/bin/env bash
    set -euo pipefail

    host="{{ host }}"
    port="{{ port }}"
    started=0

    cleanup() {
      if [[ "$started" == "1" ]]; then
        just stop
      fi
    }
    trap cleanup EXIT

    if ! curl -fsS --max-time 1 "http://${host}:${port}/health" >/dev/null 2>&1; then
      just start ""
      started=1
    else
      echo "Using existing server at http://${host}:${port}"
    fi

    bash tests/e2e/run-all-tests.sh --no-server

image-local tag="secure-linkage-service:local":
    #!/usr/bin/env bash
    set -euo pipefail

    case "$(uname -m)" in
      x86_64) arch="amd64"; platform="linux/amd64" ;;
      arm64|aarch64) arch="arm64"; platform="linux/arm64/v8" ;;
      *) echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
    esac

    rm -rf artifacts
    mkdir -p "artifacts/binaries-${arch}-"

    docker run --rm \
      --platform "$platform" \
      -e ARCH="$arch" \
      -e CARGO_TARGET_DIR=/tmp/target-linux \
      -e PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
      -v "$PWD:/work" \
      -w /work \
      "{{ rust_image }}" \
      bash -lc 'cargo build --release --locked && cp "$CARGO_TARGET_DIR/release/{{ binary }}" "artifacts/binaries-${ARCH}-/{{ binary }}"'

    docker buildx build \
      --platform "$platform" \
      --build-arg TARGETARCH="$arch" \
      --build-arg FEATURE="-" \
      --build-arg COMPONENT="{{ binary }}" \
      -t "{{ tag }}" \
      --load \
      .

logs:
    tail -f {{ logfile }}

test:
    cargo test

fmt:
    cargo fmt

lint:
    cargo clippy
