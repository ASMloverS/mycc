#!/bin/bash
exec python3 "$(dirname "$0")/tools/sync_config.py" "$@"
