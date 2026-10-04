# flutter-tizen plugin for proto

A [proto](https://moonrepo.dev/proto) WASM plugin that installs and manages
[flutter-tizen](https://github.com/flutter-tizen/flutter-tizen), the Flutter tooling for Tizen devices.

## Installation

Add the plugin to a `.prototools` file, then install a version.

```toml
[plugins.tools]
flutter-tizen = "github://a-man-called-q/proto-flutter-tizen-plugin"
```

```bash
proto install flutter-tizen
```

flutter-tizen has no pre-built archives, so the plugin clones the release tag with Git and then
runs the tool once, which clones the pinned Flutter SDK and compiles the tool. Expect the first
install to download a few gigabytes. `git` must be available on `PATH`.

Supported hosts are Linux x64, macOS (Rosetta is required on Apple Silicon), and Windows x64.

The [Tizen SDK and .NET SDK](https://github.com/flutter-tizen/flutter-tizen/blob/main/doc/install-tizen-sdk.md)
that flutter-tizen builds with are not installed by this plugin.

## Versions

flutter-tizen tags releases as `<flutter version>-tizen.<revision>`. Semver treats that as a
pre-release, which version ranges never match, so the plugin lists each release with the revision
as build metadata instead.

| Requested            | Resolves to                                      |
| -------------------- | ------------------------------------------------ |
| `latest`             | The newest release                               |
| `3.47`, `^3.44`      | The newest release in that range                 |
| `3.47.1`             | The newest Tizen revision of Flutter 3.47.1      |
| `3.47.1+tizen.1.1.1` | Exactly that release                             |
| `3.47.1-tizen.1.1.1` | The same release, written as the Git tag         |
| `3.24.1`             | An older release that was tagged without a revision |

```toml
flutter-tizen = "3.47"
```

The `upgrade` and `channel` commands of flutter-tizen move the checkout to another tag, so proto
blocks them. Change the pinned version instead.

## Configuration

```toml
[tools.flutter-tizen]
# Clone from a fork or mirror
repo-url = "https://github.com/flutter-tizen/flutter-tizen.git"
# Leave the Flutter SDK setup to the first `flutter-tizen` run
skip-bootstrap = false
```

## Contributing

The WASM target needs a rustup-managed toolchain. If `cargo` resolves to another install, such as
Homebrew, put `~/.cargo/bin` first on `PATH`.

```bash
cargo build --target wasm32-wasip1
```

Tests run against the built `.wasm` file, so build it first. They also create shims, which needs
proto itself to be installed.

```bash
cargo test --no-default-features
```

The test that sets up the Flutter SDK is ignored by default.

```bash
cargo test --no-default-features --test install_test -- --ignored
```

The `.prototools` in this repository points `flutter-tizen` at the debug build, so `proto` commands
run from here use the local plugin.

```bash
proto --log trace install flutter-tizen
```
