#!/bin/bash

wget -qO- http://localhost:3000/sh/zypp/github/mominul/pack-exp2 | sh

zypper --gpg-auto-import-keys refresh

output=$(zypper search openbangla 2>&1)
status=$?

# Print the output of the zypper command
echo "$output"

# Check if the dnf command was successful
if [ $status -ne 0 ]; then
    echo "Error: zypper search command failed." >&2
    exit $status
fi

# Check if `fcitx-openbangla` is in the output
if echo "$output" | grep -q "fcitx-openbangla"; then
    echo
    echo "Package fcitx-openbangla found."
else
    echo "Error: fcitx-openbangla not found." >&2
    exit 1
fi

# Check if `ibus-openbangla` is in the output
if echo "$output" | grep -q "ibus-openbangla"; then
    echo
    echo "Package ibus-openbangla found."
else
    echo "Error: ibus-openbangla not found." >&2
    exit 1
fi
