# Packaging

One package on every distribution: a single executable and the three files that
make it an application rather than a command.

```text
bin/songonsole
share/applications/songonsole.desktop
share/icons/hicolor/scalable/apps/songonsole.svg
share/metainfo/io.github.petexy.songonsole.metainfo.xml
```

`install.sh` stages exactly that into a `DESTDIR`, and every recipe here calls
it — which is what stops one distribution quietly shipping a different set of
files from another. It installs no licence and no documentation, because where
those go is the one thing the distributions genuinely disagree about.

## Building

```sh
./packaging/build.sh arch      # | debian | fedora | nix
./packaging/build.sh check     # what a package would have to agree with
```

Everything lands in `packaging/out`. The build directory defaults to
`packaging/out/build` rather than `$TMPDIR` on purpose: on a systemd machine
`/tmp` is a tmpfs sized at a fraction of RAM, and this is a GPU application
whose locked graph is several hundred crates. Point it elsewhere with
`--work-dir DIR` or `SONGONSOLE_WORK_DIR`.

## The toolkit is a build dependency

This is the thing about the package that looks like a mistake and is not.
`lxb-app`, `lxb-render` and `lxb-toolkit` are Rust **path** dependencies —
and `lxb-app` brings `lxb-input`, `lxb-sound` and `lxb-portal` with it — so
cargo compiles their sources into this binary: what comes out
links no `liblxb_*.so` and runs on a machine that has never heard of the
toolkit. It is versioned all the same, because this is built against one
release of the design language rather than against whichever happens to be
lying about.

The sources come from the toolkit's development component, at
`/usr/share/lxb-toolkit/crates` — which is the path `Cargo.toml` names, so it
is the same on every distribution here. `LXB_TOOLKIT_CRATE_DIR` points
somewhere else; the Nix build rewrites it to a store path.

## FFmpeg is a runtime dependency and not a build one

The other thing that looks wrong. Nothing here links `libavcodec`: `ffprobe` is
run as a *program* to read a file's tags and its length, and `ffmpeg` to lift
out the artwork a file carries. So a version bump of FFmpeg does not rebuild
this package, and no `libav*-dev` appears in any build dependency — but a
machine without the two commands has a library that never fills, so every
recipe names them as a hard runtime dependency rather than an optional one.

What *is* linked, and is easy to miss, is `alsa-lib`: the sound library opens
the machine's output through it, and `readelf -d` on the finished binary is
where that shows.

## What each recipe carries

| | |
|---|---|
| `arch/` | `PKGBUILD.in`, filled in from `VERSION` and the source archive |
| `debian/` | binary and source control files, and the copyright record |
| `fedora/` | one spec, with the licence of the whole vendored graph |
| `nix/` | a derivation that calls the same `install.sh` |

`check.sh` is what holds them together: it compares the version in every
recipe against `VERSION`, the toolkit requirement in each against what
`Cargo.toml` really asks for, and the staged payload against the list above.
