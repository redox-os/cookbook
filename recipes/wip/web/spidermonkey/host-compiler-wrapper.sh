#!/bin/bash

# This script intercepts calls to the cross-compiler for host tools
# and redirects them to the host compiler

# Extract base command from full path
BASENAME=$(basename "$0")

# If it's a cross-compiler call (for x86_64-unknown-redox)
if [[ "$BASENAME" == *"x86_64-unknown-redox"* ]]; then
    # Extract compiler type (gcc, g++, etc.)
    COMPILER="${BASENAME##*-}"
    
    # Check if we're building a host tool
    if [[ "$*" == *"host_"* || "$*" == *"nsinstall"* ]]; then
        echo "REDIRECTING cross-compiler call to host compiler for host tools" >&2
        # Use the native compiler instead
        HOST_COMPILER="$COMPILER"
        # Remove sysroot flags that would be for the target system
        ARGS=()
        SKIP_NEXT=0
        for ARG in "$@"; do
            if [[ $SKIP_NEXT -eq 1 ]]; then
                SKIP_NEXT=0
                continue
            fi
            if [[ "$ARG" == "--sysroot" ]]; then
                SKIP_NEXT=1
                continue
            fi
            if [[ "$ARG" == "-isystem" ]]; then
                SKIP_NEXT=1
                continue
            fi
            if [[ "$ARG" == "-L/"* || "$ARG" == "-I/"* ]]; then
                if [[ "$ARG" == *"/x86_64-unknown-redox/"* ]]; then
                    continue
                fi
            fi
            ARGS+=("$ARG")
        done
        
        echo "Running $HOST_COMPILER ${ARGS[@]}" >&2
        exec "$HOST_COMPILER" "${ARGS[@]}"
    fi
fi

# Pass through to the original command if not redirected
exec "$0.real" "$@"