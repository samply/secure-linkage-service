#!/usr/bin/env bash

# Destroy the current virtual environment
rm -rf ./venv

# Create the virtual environment
python3 -m venv ./venv

# Activate the virtual environment
source venv/bin/activate

# Install the dependencies
pip install -r requirements.txt

# Start the server in development mode (without TLS)
LS_NO_TLS=true python3 src/main.py

# Deactivate the virtual environment
deactivate
