Name:           songonsole
Version:        0.9.1
Release:        1%{?dist}
Summary:        A music library and player in the LineXinBar design language, shown as Music

# This program, and the locked Rust dependency graph vendored into the source
# archive. Every crate offering a choice is taken under its permissive option:
# self_cell as Apache-2.0 rather than GPL-2.0-only, r-efi as MIT rather than
# LGPL-2.1-or-later. What is left after that choice is this list, and it is
# derived from the lock file rather than remembered.
License:        GPL-3.0-only AND Apache-2.0 AND MIT AND Apache-2.0 WITH LLVM-exception AND BSD-2-Clause AND BSD-3-Clause AND ISC AND MPL-2.0 AND Unicode-3.0 AND Unlicense AND Zlib AND 0BSD AND CDLA-Permissive-2.0
URL:            https://github.com/Petexy/songonsole
Source0:        songonsole-%{version}.tar.gz

ExclusiveArch:  x86_64 aarch64

# Cargo's release profile emits no DWARF, so find-debuginfo would produce an
# empty debugsourcefiles.list and rpmbuild would fail on it after the whole
# build. An archive submission wants real debuginfo instead: drop this, and
# with it the -Cdebuginfo=0 in %build that holds Fedora's own -Cdebuginfo=2 off,
# so the DWARF is built and packaged rather than built and binned.
%global debug_package %{nil}

BuildRequires:  cargo >= 1.90
BuildRequires:  rust >= 1.90
BuildRequires:  gcc
BuildRequires:  pkgconfig
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib
# The design language, as Rust sources. It is a build dependency and not a
# runtime one: `lxb-app` is a path dependency, so cargo compiles it into this
# binary and the finished program links no liblxb_*.so at all.
BuildRequires:  lxb-toolkit-devel >= 0.9.1
# What the program links outright, each asked for as a pkg-config name, which
# is what the Rust bindings look for: ALSA for the interface sounds, libudev
# for the game controllers and xkbcommon for the keyboard.
BuildRequires:  pkgconfig(alsa)
BuildRequires:  pkgconfig(libudev)
BuildRequires:  pkgconfig(xkbcommon)

# Opened by name at run time rather than linked, so rpm's automatic dependency
# generator cannot see it in the ELF.
Requires:       libglvnd-egl
# Choosing another folder is put to whatever chooser this desktop runs, through
# the portal. Without one the toolkit draws its own, so this is not required.
Recommends:     xdg-desktop-portal
# With a Vulkan driver present this draws through it; without one it falls back
# to EGL, so the loader is worth having and is not required.
Suggests:       vulkan-loader
# What reads a file's tags, its length and the artwork inside it. `ffprobe` and
# `ffmpeg` are run as *programs* — nothing links libavcodec — so rpm cannot see
# them in the ELF either, and a machine without them has a library that stays
# empty. Fedora ships two ffmpegs that both provide the command, the free build
# in the main repository and the whole one from RPM Fusion; either answers.
Requires:       /usr/bin/ffprobe
Requires:       /usr/bin/ffmpeg
# And %%check's: the media tests make a tagged file of each supported kind with
# ffmpeg and read it back through ffprobe, the way the library does.
BuildRequires:  /usr/bin/ffprobe
BuildRequires:  /usr/bin/ffmpeg

%description
Shown as Music. A library and a player drawn in the LineXinBar design language:
the same colours, glass, motion and marks as the shell it was made for, and
driven from a controller, a keyboard and a pointer at once. It is an ordinary
Wayland application and runs under GNOME or Plasma as readily as under that
shell.

Your songs are gathered from the folders you name, read for their tags and
their artwork, and laid out five ways: every song by name, a wall of record
sleeves, a shelf of artists, the ones you have kept, and the queue. Pressing a
song grows its sleeve out of the row it was on into a Now Playing page that
fills the window — the artwork as large as the screen will allow, the song, the
artist, the album it came from, where it has got to, and what is coming after
it. Back shrinks that page into the row it came from, and the music keeps
playing while you go on browsing.

The transport is the same row of controls on both screens, so shuffle, repeat,
skipping, seeking and the volume are always in the same place; the row along
the bottom of the library is both what is playing and how to control it.

Tags, lengths and embedded artwork are read with ffprobe and ffmpeg, which are
run as programs rather than linked. Artwork lifted out of a file is kept in the
usual cache so it is only ever read once.

%prep
%autosetup -n songonsole-%{version}

%build
export RUSTUP_TOOLCHAIN=stable
export CARGO_TARGET_DIR=target
# Fedora exports its own %%{build_rustflags} into RUSTFLAGS before this runs, and
# they carry -Cdebuginfo=2 -Cstrip=none. RUSTFLAGS is appended after the release
# profile's own flags and wins, so every crate in the graph was generating full
# DWARF — and with %%global debug_package %%{nil} above, no package was ever made
# of it. -Cdebuginfo=0 last is what turns that back off. It is worth about
# 274 MiB of resident memory on the final rustc here, measured: 1572 MiB with
# the DWARF, 1298 MiB without.
export RUSTFLAGS="${RUSTFLAGS:-} -Cdebuginfo=0"

# And Cargo takes its job count from the core count alone, knowing nothing about
# how much memory the machine has to hold that many rustc at once. wgpu and naga
# are in this graph and thin LTO with one codegen unit is what the release
# profile asks for, so the count has to answer to memory as well. The sister
# repository's shell was killed by the kernel's OOM killer twice on an 8 GiB
# Apple M1 for want of exactly this.
#
# Arithmetic rather than %%limit_build, the Fedora macro meant for this, which
# swallowed the remainder of the script it was used in on Fedora Asahi.
build_jobs="%{_smp_build_ncpus}"
build_room="$(awk '/^MemTotal:/ { n = int($2 / 1024 / 2048); print (n < 1 ? 1 : n) }' /proc/meminfo 2>/dev/null || true)"
if [ -n "$build_room" ] && [ "$build_room" -lt "$build_jobs" ]; then
    build_jobs="$build_room"
fi
echo "building with $build_jobs of %{_smp_build_ncpus} jobs, for the memory this machine has"
cargo build --offline --locked --release -j"$build_jobs"

%install
export CARGO_TARGET_DIR=target
./packaging/install.sh \
    --destdir %{buildroot} \
    --prefix %{_prefix} \
    --target-dir target

%check
export RUSTUP_TOOLCHAIN=stable
export CARGO_TARGET_DIR=target
# The same two as %%build. The dev profile asks for full DWARF and this phase
# builds the graph a second time to get it, with no package made of it either;
# a failing test still names its file and line, which the panic carries rather
# than DWARF.
export RUSTFLAGS="${RUSTFLAGS:-} -Cdebuginfo=0"
build_jobs="%{_smp_build_ncpus}"
build_room="$(awk '/^MemTotal:/ { n = int($2 / 1024 / 2048); print (n < 1 ? 1 : n) }' /proc/meminfo 2>/dev/null || true)"
if [ -n "$build_room" ] && [ "$build_room" -lt "$build_jobs" ]; then
    build_jobs="$build_room"
fi
cargo test --offline --locked -j"$build_jobs"
# The two files that are read by something other than this program. Both are
# installed by then, so what is checked is what ships rather than what is in
# the checkout.
desktop-file-validate %{buildroot}%{_datadir}/applications/songonsole.desktop
appstream-util validate-relax --nonet \
    %{buildroot}%{_metainfodir}/io.github.petexy.songonsole.metainfo.xml

%files
%license LICENSE
%doc README.md
%{_bindir}/songonsole
%{_datadir}/applications/songonsole.desktop
%{_datadir}/icons/hicolor/scalable/apps/songonsole.svg
%{_metainfodir}/io.github.petexy.songonsole.metainfo.xml

%changelog
* Thu Sep 24 2026 Piotr Lewandowski <piotr.petexy@gmail.com> - 0.9.1-1
- Released with LineXinBar 0.9.1. Sorting is one row of the Options menu, with
  the six orders behind it.
- The transport lays out its chips and marks at the size the window really is,
  so two controls no longer share one place in a small window.
- The rail's Options chip is gone; the legend already names that button.
- --shot takes --width and --height, as the other three applications do, and
  the Makefile has given way to a flake at the root, as theirs has.
- The wallpaper carries the shell's sparkles and follows Theme > Particles,
  which lxb-app hands every window.
- Requires lxb-toolkit 0.9.1 to build, the version the family releases under.

* Thu Sep 18 2026 Piotr Lewandowski <piotr.petexy@gmail.com> - 0.9.0-1
- First packaged release. A music library and one song at a time, in the
  LineXinBar design language.
- Five ways into a library — songs, albums, artists, favourites and the queue —
  and a Now Playing page that grows out of the record sleeve that was pressed
  and shrinks back into it.
- One transport on both screens: shuffle, previous, play, next, repeat, seek
  either way and the volume, always in the same place.
- Tags, lengths and embedded artwork are read with ffprobe and ffmpeg; the
  sound is played through ALSA, and so through PipeWire or PulseAudio where the
  machine runs one.
- Ten languages, from the same catalogues the rest of the family speaks.
- Requires lxb-toolkit 0.9.0 to build, the version LineXinBar, the toolkit,
  Imagonsole, Videonsole, DistriBumpy and CEDM all release under.
