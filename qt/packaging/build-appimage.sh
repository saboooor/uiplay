#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_dir="${repo_root}/qt/build-appimage"
app_dir="${build_dir}/AppDir"
tools_dir="${build_dir}/tools"

cmake -S "${repo_root}/qt" -B "${build_dir}" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=/usr
cmake --build "${build_dir}" --parallel
DESTDIR="${app_dir}" cmake --install "${build_dir}"

mkdir -p "${tools_dir}"
curl --fail --location --retry 3 \
  --output "${tools_dir}/linuxdeploy.AppImage" \
  https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
curl --fail --location --retry 3 \
  --output "${tools_dir}/linuxdeploy-plugin-qt" \
  https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-x86_64.AppImage
chmod +x "${tools_dir}/linuxdeploy.AppImage" "${tools_dir}/linuxdeploy-plugin-qt"
cp "${repo_root}/src-tauri/icons/icon.png" "${tools_dir}/ca.saboor.uiplay.png"

export QML_SOURCES_PATHS="${repo_root}/qt/qml"
export OUTPUT="${build_dir}/UiPlay-1.0.0-alpha.1-x86_64.AppImage"
export PATH="${tools_dir}:${PATH}"
export APPIMAGE_EXTRACT_AND_RUN=1

cd "${repo_root}"
"${tools_dir}/linuxdeploy.AppImage" --appimage-extract-and-run \
  --appdir "${app_dir}" \
  --desktop-file "${repo_root}/qt/packaging/ca.saboor.uiplay.desktop" \
  --icon-file "${tools_dir}/ca.saboor.uiplay.png" \
  --plugin qt \
  --output appimage

printf '%s\n' "${OUTPUT}"
