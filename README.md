<img width="128" height="128" alt="image" src="https://raw.githubusercontent.com/saboooor/uiplay/refs/heads/master/src-tauri/icons/icon.png" />

# UiPlay

A Tauri and Qwik based UxPlay wrapper that forwards music metadata to Discord Rich Presence and MPRIS.

<img height="600" alt="image" src="https://raw.githubusercontent.com/saboooor/uiplay/refs/heads/master/screenshots/app.png" />
<img height="300" alt="image" src="https://raw.githubusercontent.com/saboooor/uiplay/refs/heads/master/screenshots/activity.png" />

## Building

To build the project, ensure you have the necessary dependencies installed:

- [Bun](https://bun.sh/)
- [Rust](https://rust-lang.org/)
- [UxPlay](https://github.com/FDH2/uxplay)
- `avahi-browse` (usually provided by `avahi-utils`) for UxPlay media controls

Then, run the following commands in the project root:

```bash
bun install
bun tauri build
```

### Native Qt application

An in-progress native Qt 6 / Qt Quick port lives in [`qt/`](qt/). It can be
built without Bun, Qwik, or Tauri:

```bash
cmake -S qt -B qt/build -DCMAKE_BUILD_TYPE=Release
cmake --build qt/build
./qt/build/uiplay-qt
```

See [`qt/README.md`](qt/README.md) for its current feature-parity status.

## Static Site Generator (Node.js)

Be sure to configure your server to serve very long cache headers for the `build/**/*.js` files.

Typically you'd set the `Cache-Control` header for those files to `public, max-age=31536000, immutable`.

```shell
pnpm build.server
```
