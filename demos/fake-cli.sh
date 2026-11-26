#!/bin/sh
# Deterministic stand-in for game-recomp-cli used only by SDK demos.
case "$1" in
  inspect)
    printf '{"machine":"i386","pe_type":"PE32"}\n'
    ;;
  run)
    out=""
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "--project-dir" ]; then out="$2"; fi
      shift
    done
    [ -n "$out" ] || exit 2
    mkdir -p "$out/reports"
    printf '{"schema":1,"score":42,"stages":["inspect","recover","translate","project","compile","run","verify","report"]}\n' > "$out/reports/report.json"
    ;;
  *) exit 2 ;;
esac
exit 0
