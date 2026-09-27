#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "-query" && "${2:-}" == "QT_INSTALL_PLUGINS" ]]; then
  printf '%s\n' "${UIPLAY_QT_PLUGIN_PATH:?}"
  exit 0
fi

if [[ "${1:-}" == "-query" && $# -eq 1 ]]; then
  "${UIPLAY_REAL_QMAKE:?}" -query | sed \
    "s|^QT_INSTALL_PLUGINS:.*$|QT_INSTALL_PLUGINS:${UIPLAY_QT_PLUGIN_PATH:?}|"
  exit 0
fi

exec "${UIPLAY_REAL_QMAKE:?}" "$@"
