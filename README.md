# secure-linkage-service


## Requirements

* Python 3.13+
* python3-venv (built-in into Python since Python 3.3)


## Setup

The server runs in a Virtual Environment (via `venv`).

```shell
# Create the virtual environment
python3 -m venv ./venv

# Activate the virtual environment
source venv/bin/activate

# Install the dependencies
pip install -r requirements.txt

# Once you are done,
# you should deactivate the virtual environment
deactivate
```

## Usage

This assumes the system has already been setup (see previous step).

```shell
# Activate the virtual environment
# (if not already inside it)
source venv/bin/activate

# Run the unit tests
pytest

# Start the server in development mode (without TLS)
LS_NO_TLS=true python3 src/main.py

...

# Once you are done,
# you should deactivate the virtual environment
deactivate
```

## Example Scenario: Trying a Linkage

#### Description

0. Re-create and activate the virtual environment.
1. Start the server.
2. Send a request with a *server* role.
3. Within 20 seconds (default timeout), send a request with a *client* role.
4. Stop the server.
5. Disable the virtual environment.

#### E2E Testing

   ```sh
   ./run-e2e-tests.sh
   ```

## Troubleshooting / Known Problems

### E2E testing

* Make sure all scripts are executable:
   ```sh
   chmod +x *.sh
   ```
* There is bug in the E2E script that prevents it from successfully killing the server process.
You will have to do it manually, via `kill`.

### Server Restart

* In some occasions, when restarting the server, you may get an import error.

```python3
ImportError: cannot import name 'generate_etag' from partially initialized module 'werkzeug.http' (most likely due to a circular import)
```
Just re-create the virtual environment and try again.
The script `start-server.sh` takes care of this.


## License

Copyright © since 2023, Tobias Kussel; since 2025, Leandro Doctors.

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

You should have received a copy of the GNU Affero General Public License along
with this program. If not, see http://www.gnu.org/licenses/.
