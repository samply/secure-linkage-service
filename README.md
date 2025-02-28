# securelinkageservice


## Requirements

* Python 3.7+
* python3-venv (built-in into Python since Python 3.3)


## Setup

The server runs in a Virtual Environment (via `venv`).

```shell
# Create the virtual environment
python3 -m venv ./venv

# Activate the virtual environment
source venv/bin/activate

# Install the dependencies
pip3 install -r requirements.txt

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

# Run the tests
pytest

# Start the server in development mode (without TLS)
LS_NO_TLS=true python3 src/main.py

...

# Once you are done,
# you should deactivate the virtual environment
deactivate
```


## License

Copyright © since 2023, Tobias Kussel; since 2025, Leandro Doctors.

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

You should have received a copy of the GNU Affero General Public License along
with this program. If not, see http://www.gnu.org/licenses/.
