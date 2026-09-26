# UiPlay Qt

Native Qt 6 / Qt Quick port of UiPlay. The existing Tauri application is kept
alongside it while feature parity is completed.

## Build

```sh
cmake -S qt -B qt/build -DCMAKE_BUILD_TYPE=Release
cmake --build qt/build
./qt/build/uiplay-qt
```

The application expects either `shairport-sync` or `uxplay` to be installed.
Shairport metadata, album art, receiver lifecycle, settings, device state, logs,
and its D-Bus playback controls are handled by the C++ backend. Window borders,
the title bar, and window controls are supplied by the native window manager.
On KDE Plasma, UiPlay selects KDE's Qt Quick Controls desktop style when it is
installed; elsewhere it follows the active Qt platform style. Settings and the
receiver terminal are native modal dialogs rather than application pages.

For development builds, album-art CDN uploads read `APP_SECRET` from
`src-tauri/.env`. Packaged builds should provide the same value through the
`UIPLAY_APP_SECRET` environment variable. The secret is never written to the
Qt settings file or application logs.

## Native integrations

- MPRIS metadata, progress, playback state, and controls.
- Discord Rich Presence with track timestamps and remote artwork.
- Authenticated album-art upload for Discord and MPRIS consumers.
- DACP controls for Shairport Sync and UxPlay, with Shairport D-Bus fallback.
- AppImage-safe UxPlay environment handling.
- Freedesktop application metadata and icon installation.

## Packages

The release workflow produces three Linux formats for version
`1.0.0-alpha.1`:

- `UiPlay-1.0.0-alpha.1-x86_64.AppImage`
- `uiplay-qt_1.0.0~alpha1_amd64.deb`
- `UiPlay-1.0.0-alpha.1-x86_64.flatpak`

Create a `qt-v1.0.0-alpha.1` tag to publish them together as a GitHub
prerelease. Branch, pull-request, and manually dispatched builds upload the
same files as workflow artifacts without creating a release. Flatpak launches
the host's installed `shairport-sync` or `uxplay` through
`flatpak-spawn --host`; at least one receiver must therefore still be installed
on the host.
