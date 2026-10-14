set shell := ["bash", "-cu"]

binary := "secure-linkage-service"
host := "localhost"
port := "5000"
pidfile := ".server.pid"
logfile := "server.log"

_default:
    @just --list

build:
    cargo build --release

start:
    #!/usr/bin/env bash
    set -euo pipefail

    binary="{{ binary }}"
    host="{{ host }}"
    port="{{ port }}"
    pidfile="{{ pidfile }}"
    logfile="{{ logfile }}"

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
      just start
      started=1
    else
      echo "Using existing server at http://${host}:${port}"
    fi

    bash tests/e2e/run-all-tests.sh

logs:
    tail -f {{ logfile }}

test:
    cargo test

fmt:
    cargo fmt

lint:
    cargo clippy
