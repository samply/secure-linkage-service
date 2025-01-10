# securelinkageservice

## Requirements

* Python 3 (currently tested on Python 3.11; should run on Python 3.7+)
* pipenv

## Running the server

### Using `pipenv`

```shell
# Install the dependencies (automatically creates the virtual environment if needed)
pipenv install

# Starting the daemon (without setting up TLS)

## Directly from the normal shell
LS_NO_TLS=true pipenv run python3 main.py

## From within the virtual environment
pipenv shell
LS_NO_TLS=true python3 main.py

...

exit

...

# Remove the virtual environment
pipenv --rm

```


## License

Copyright © since 2023, Tobias Kussel; since 2025, Leandro Doctors.

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

You should have received a copy of the GNU Affero General Public License along
with this program. If not, see http://www.gnu.org/licenses/.
