#!/usr/bin/env bash

# Check if the arguments are provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <profile>"
    exit 1
fi

# Assign arguments to variables
profile=$1

# Pass the contract name to the sncast command
sncast --profile "$profile" --wait declare --contract-name SharpP2PRegistry