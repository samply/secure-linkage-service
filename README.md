# securelinkageservice

## Requirements

* Python 3.7+
* python3-venv (built-in into Python since Python 3.3)


## Running the server

### Using `venv`

```shell
# Create a virtual environment to isolate the project denpendencies
python3 -m venv ./venv

# Activate it
source venv/bin/activate

# Install the dependencies
pip3 install -r requirements.txt

# Start the daemon (without setting up TLS)
LS_NO_TLS=true python3 main.py

...

# After execution, deactivate the virtual environment
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
