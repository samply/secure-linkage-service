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

### Trying a Linkage

1. Start the server (see above).
2. From another terminal, send a request with a *server* role (example below).
3. Within 20 seconds (default timeout), send a request with a *client* role (example below).


## Examples

### Server request

```shell
curl -v http://localhost:5000/linkageResult/dkfz/tuda \
     -H "Content-Type: application/json" \
     -d '{
       "role": "server",
       "result": [
         {
           "match": 0,
           "tentativeMatch": 1,
           "bestIndex": 2
         },
         {
           "match": 1,
           "tentativeMatch": 0,
           "bestIndex": 0
         },
         {
           "match": 0,
           "tentativeMatch": 0,
           "bestIndex": 4
         }
       ],
       "ids": ["aGVsbG93b3JsZA==", "ZGF0YXNjaWVuY2U=", "cHl0aG9uaXNmdW4=", "cHJpdmFjeXByZXNlcnZpbmc=", "bGlua2FnZXN5c3RlbQ=="]
     }'
* Host localhost:5000 was resolved.
* IPv6: ::1
* IPv4: 127.0.0.1
*   Trying [::1]:5000...
* connect to ::1 port 5000 from ::1 port 50908 failed: Connection refused
*   Trying 127.0.0.1:5000...
* Connected to localhost (127.0.0.1) port 5000
* using HTTP/1.x
> POST /linkageResult/dkfz/tuda HTTP/1.1
> Host: localhost:5000
> User-Agent: curl/8.12.1
> Accept: */*
> Content-Type: application/json
> Content-Length: 500
>
* upload completely sent off: 500 bytes
< HTTP/1.1 200 OK
< Server: Werkzeug/3.1.3 Python/3.13.2
< Date: Fri, 28 Feb 2025 16:10:23 GMT
< Content-Type: text/html; charset=utf-8
< Content-Length: 22
< Connection: close
<
* shutting down connection #0
Received server result%
```

### Client request

```shell
curl -v http://localhost:5000/linkageResult/tuda/dkfz \
     -H "Content-Type: application/json" \
     -d '{
       "role": "client",
       "result": [
         {
           "match": 1,
           "tentativeMatch": 0,
           "bestIndex": 2
         },
         {
           "match": 0,
           "tentativeMatch": 1,
           "bestIndex": 0
         },
         {
           "match": 1,
           "tentativeMatch": 1,
           "bestIndex": 4
         }
       ]
     }'
* Host localhost:5000 was resolved.
* IPv6: ::1
* IPv4: 127.0.0.1
*   Trying [::1]:5000...
* connect to ::1 port 5000 from ::1 port 50910 failed: Connection refused
*   Trying 127.0.0.1:5000...
* Connected to localhost (127.0.0.1) port 5000
* using HTTP/1.x
> POST /linkageResult/tuda/dkfz HTTP/1.1
> Host: localhost:5000
> User-Agent: curl/8.12.1
> Accept: */*
> Content-Type: application/json
> Content-Length: 372
>
* upload completely sent off: 372 bytes
< HTTP/1.1 200 OK
< Server: Werkzeug/3.1.3 Python/3.13.2
< Date: Fri, 28 Feb 2025 16:10:26 GMT
< Content-Type: text/html; charset=utf-8
< Content-Length: 141
< Connection: close
<
* shutting down connection #0
{"result": [{"linkageId": "+g5vQbykaWQZRr0/W5Dx04k="}, {"linkageId": "w2lC/gngZrveiIsB5wxgBxQ="}, {"linkageId": "7ckxg4tR7msYW0TbB7E1KZc="}]}%
```


## License

Copyright © since 2023, Tobias Kussel; since 2025, Leandro Doctors.

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

You should have received a copy of the GNU Affero General Public License along
with this program. If not, see http://www.gnu.org/licenses/.
