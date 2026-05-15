#!/usr/bin/env bash
set -euo pipefail

args=()
for arg in "$@"; do
    case "${arg}" in
        --target=*) ;;
        *) args+=("${arg}") ;;
    esac
done

exec zig cc -target x86_64-linux-gnu "${args[@]}"
