#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_dir="${repo_root}/qt/build-pacman"
package_root="${build_dir}/pkg"
version="1.0.0_alpha"
architecture="${CARCH:-$(uname -m)}"
output="${build_dir}/uiplay-qt-${version}-1-${architecture}.pkg.tar.zst"

cmake -E remove_directory "${package_root}"
cmake -S "${repo_root}/qt" -B "${build_dir}" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=/usr
cmake --build "${build_dir}" --parallel
DESTDIR="${package_root}" cmake --install "${build_dir}"

installed_size="$(du -sb "${package_root}" | cut -f1)"
build_date="${SOURCE_DATE_EPOCH:-$(date +%s)}"
printf '%s\n' \
  'pkgname = uiplay-qt' \
  "pkgbase = uiplay-qt" \
  "pkgver = ${version}-1" \
  'pkgdesc = Native AirPlay receiver with MPRIS and Discord Rich Presence' \
  'url = https://github.com/LuminescentDev/UiPlay' \
  "builddate = ${build_date}" \
  'packager = UiPlay GitHub Actions' \
  "size = ${installed_size}" \
  "arch = ${architecture}" \
  'license = custom' \
  'depend = qt6-base' \
  'depend = qt6-declarative' \
  'depend = qt6-shadertools' \
  'optdepend = shairport-sync: AirPlay audio receiver' \
  'optdepend = uxplay: AirPlay mirroring receiver' \
  'optdepend = libpulse: live audio visualizer support' \
  > "${package_root}/.PKGINFO"

tar --zstd -C "${package_root}" -cf "${output}" .PKGINFO usr
printf '%s\n' "${output}"
