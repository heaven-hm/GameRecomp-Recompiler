#!/bin/sh
# Thin adapter around the REAL game-recomp-cli v0.3.0 binary.
# The SDK's fixed contract cannot pass --skip-stages; verification is fail-closed
# without a Windows oracle, so this wrapper appends that flag.
set -eu
: "${GAMERECOMP_REAL_CLI:?set GAMERECOMP_REAL_CLI to the real game-recomp-cli path}"
: "${GAMERECOMP_SOURCE_ROOT:?set GAMERECOMP_SOURCE_ROOT to the GameReComp checkout}"
cd "$GAMERECOMP_SOURCE_ROOT"
export PYTHONPATH="$GAMERECOMP_SOURCE_ROOT${PYTHONPATH:+:$PYTHONPATH}"
exec "$GAMERECOMP_REAL_CLI" "$@" --skip-stages verify
