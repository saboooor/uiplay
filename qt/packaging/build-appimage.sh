#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_dir="${repo_root}/qt/build-appimage"
app_dir="${build_dir}/AppDir"
tools_dir="${build_dir}/tools"

if [[ -z "${QMAKE:-}" ]]; then
  if command -v qmake6 >/dev/null 2>&1; then
    QMAKE="$(command -v qmake6)"
  elif command -v qmake-qt6 >/dev/null 2>&1; then
    QMAKE="$(command -v qmake-qt6)"
  elif command -v qmake >/dev/null 2>&1 \
       && [[ "$(qmake -query QT_VERSION 2>/dev/null)" == 6.* ]]; then
    QMAKE="$(command -v qmake)"
  else
    printf '%s\n' "Unable to find a Qt 6 qmake. Set QMAKE to the Qt 6 qmake executable." >&2
    exit 1
  fi
fi

if [[ "$("${QMAKE}" -query QT_VERSION 2>/dev/null)" != 6.* ]]; then
  printf 'QMAKE must point to Qt 6, but %s reports version %s\n' \
    "${QMAKE}" "$("${QMAKE}" -query QT_VERSION 2>/dev/null || printf unknown)" >&2
  exit 1
fi
export QMAKE

# Use a private plugin tree so distro-specific optional codecs cannot make the
# package fail because one of their non-Qt dependencies is missing. Standard
# Qt image format plugins are retained; only KDE's optional kimg_* plugins are
# omitted.
qt_plugin_source="$("${QMAKE}" -query QT_INSTALL_PLUGINS)"
qt_plugin_stage="${tools_dir}/qt-plugins"
mkdir -p "${qt_plugin_stage}"
cp -a "${qt_plugin_source}/." "${qt_plugin_stage}/"
for optional_plugin in "${qt_plugin_stage}"/imageformats/kimg_*.so; do
  if [[ -e "${optional_plugin}" ]]; then
    rm -f -- "${optional_plugin}"
  fi
done

export UIPLAY_REAL_QMAKE="${QMAKE}"
export UIPLAY_QT_PLUGIN_PATH="${qt_plugin_stage}"
export QMAKE="${repo_root}/qt/packaging/qmake-appimage-wrapper.sh"

# Never allow files from an earlier deployment attempt to leak into the next
# AppImage.
cmake -E remove_directory "${app_dir}"

cmake -S "${repo_root}/qt" -B "${build_dir}" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=/usr
cmake --build "${build_dir}" --parallel
DESTDIR="${app_dir}" cmake --install "${build_dir}"
cp "${repo_root}/qt/packaging/AppRun" "${app_dir}/AppRun"
chmod +x "${app_dir}/AppRun"

mkdir -p "${tools_dir}"
if [[ ! -x "${tools_dir}/linuxdeploy.AppImage" ]]; then
  curl --fail --location --retry 3 \
    --output "${tools_dir}/linuxdeploy.AppImage" \
    https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
fi
if [[ ! -x "${tools_dir}/linuxdeploy-plugin-qt" ]]; then
  curl --fail --location --retry 3 \
    --output "${tools_dir}/linuxdeploy-plugin-qt" \
    https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-x86_64.AppImage
fi
chmod +x "${tools_dir}/linuxdeploy.AppImage" "${tools_dir}/linuxdeploy-plugin-qt"
cp "${repo_root}/src-tauri/icons/icon.png" "${tools_dir}/ca.saboor.uiplay.png"

export QML_SOURCES_PATHS="${repo_root}/qt/qml"
export OUTPUT="${build_dir}/UiPlay-1.0.0-alpha-x86_64.AppImage"
export PATH="${tools_dir}:${PATH}"
export APPIMAGE_EXTRACT_AND_RUN=1
# linuxdeploy's bundled binutils cannot strip newer RELR-enabled binaries
# produced by rolling-release distributions such as Arch Linux.
export NO_STRIP=1

cd "${repo_root}"
"${tools_dir}/linuxdeploy.AppImage" --appimage-extract-and-run \
  --appdir "${app_dir}" \
  --desktop-file "${repo_root}/qt/packaging/ca.saboor.uiplay.desktop" \
  --icon-file "${tools_dir}/ca.saboor.uiplay.png" \
  --plugin qt \
  --output appimage

printf '%s\n' "${OUTPUT}"
