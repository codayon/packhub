#!/bin/bash

apt update
apt install sudo -y

wget -qO- http://host.docker.internal:3000/sh/github/ubuntu/mominul/pack-exp3 | sh

output=$(apt search openbangla 2>&1)
status=$?

# Print the output of the apt command
echo "$output"

# Check if the apt command was successful
if [ $status -ne 0 ]; then
    echo "Error: apt search command failed." >&2
    exit $status
fi

if echo "$output" | grep -q "openbangla-keyboard"; then
    echo
    echo "Package found successfully."
    exit 0
else
    echo "Error: package not found." >&2
    exit 1
fi
