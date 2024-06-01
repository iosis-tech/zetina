#!/usr/bin/env bash

# Check if the correct number of arguments are provided
if [ $# -ne 2 ]; then
    echo "Usage: $0 <profile> <class-hash>"
    exit 1
fi

# Assign arguments to variables
profile=$1
class_hash=$2

# Pass the class hash to the sncast command
sncast --profile "$profile" --wait deploy --class-hash "$class_hash" -c 0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d 0x06BE2FEB774226FDa7292189443eDD118dE9eaaFd78D97c58ae09569269b0D34 0x0 0x0 0x01172c7024f026c9bf89b47e39be72f5ed7713982f6ddc3e38976a769ab997ad
