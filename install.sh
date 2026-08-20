#!/bin/sh
# Cupid installer for Linux and macOS.
#
#   curl -fsSL https://raw.githubusercontent.com/ShearesWeb/cupid/main/install.sh | sh
#
# Environment:
#   CUPID_VERSION        install a specific version (e.g. 0.2.1) instead of latest
#   CUPID_PREFIX         Linux install prefix (default: ~/.local)
#   CUPID_GITHUB_TOKEN   optional; also read from GITHUB_TOKEN, GH_TOKEN or
#                        `gh auth token`. Only needed to lift the anonymous
#                        GitHub API rate limit, or to install from a private fork.

set -eu

REPO="ShearesWeb/cupid"
API="https://api.github.com/repos/$REPO"

die()  { printf 'cupid: %s\n' "$*" >&2; exit 1; }
info() { printf '  %s\n' "$*"; }

command -v curl >/dev/null 2>&1 || die "curl is required"
command -v tar  >/dev/null 2>&1 || die "tar is required"

# --- token ------------------------------------------------------------------

# Anonymous requests are enough for a public repo; a token only raises the
# API rate limit (60/hour per IP) and unlocks private forks.
TOKEN="${CUPID_GITHUB_TOKEN:-${GITHUB_TOKEN:-${GH_TOKEN:-}}}"
if [ -z "$TOKEN" ] && command -v gh >/dev/null 2>&1; then
  TOKEN=$(gh auth token 2>/dev/null || true)
fi

curl_auth() {
  if [ -n "$TOKEN" ]; then
    curl -fsSL -H "Authorization: Bearer $TOKEN" "$@"
  else
    curl -fsSL "$@"
  fi
}

# --- release ----------------------------------------------------------------

if [ -n "${CUPID_VERSION:-}" ]; then
  tag="v${CUPID_VERSION#v}"
  release=$(curl_auth "$API/releases/tags/$tag") \
    || die "cannot read release $tag — check the version exists at github.com/$REPO/releases"
else
  release=$(curl_auth "$API/releases/latest") \
    || die "cannot reach the GitHub API for $REPO — check your network, or retry if rate-limited"
fi

version=$(printf '%s\n' "$release" | sed -n 's/^  "tag_name": "v\{0,1\}\([^"]*\)".*/\1/p' | head -n1)
[ -n "$version" ] || die "could not read the version out of the release"

# Assets are pulled through the API URL rather than browser_download_url: it
# works anonymously and still honours a token, so private forks work unchanged.
asset_url() {
  printf '%s\n' "$release" | awk -v want="$1" '
    /"url": "https:\/\/api\.github\.com\/.*\/releases\/assets\/[0-9]*"/ {
      u = $0; sub(/^.*"url": "/, "", u); sub(/".*$/, "", u); next
    }
    index($0, "\"name\": \"" want "\"") { print u; exit }
  '
}

download() { # url dest
  if [ -n "$TOKEN" ]; then
    curl -fL --progress-bar -H "Authorization: Bearer $TOKEN" \
      -H "Accept: application/octet-stream" "$1" -o "$2"
  else
    curl -fL --progress-bar -H "Accept: application/octet-stream" "$1" -o "$2"
  fi
}

# --- platform ---------------------------------------------------------------

os=$(uname -s)
arch=$(uname -m)

case "$os" in
  Darwin)
    case "$arch" in
      arm64|aarch64) asset="Cupid_aarch64.app.tar.gz" ;;
      *) die "only Apple Silicon builds are published; $arch Macs must build from source" ;;
    esac
    ;;
  Linux)
    case "$arch" in
      x86_64|amd64) asset="Cupid_${version}_amd64.AppImage" ;;
      *) die "only x86_64 Linux builds are published; $arch must build from source" ;;
    esac
    ;;
  *)
    die "unsupported system $os — on Windows, run the .exe installer from the releases page"
    ;;
esac

url=$(asset_url "$asset")
[ -n "$url" ] || die "release v$version has no asset named $asset"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

printf 'Installing Cupid %s (%s %s)\n' "$version" "$os" "$arch"
download "$url" "$tmp/$asset"

# --- install ----------------------------------------------------------------

install_macos() {
  tar -xzf "$tmp/$asset" -C "$tmp"
  [ -d "$tmp/Cupid.app" ] || die "the archive did not contain Cupid.app"

  dest="/Applications"
  [ -w "$dest" ] || { dest="$HOME/Applications"; mkdir -p "$dest"; }

  rm -rf "$dest/Cupid.app"
  mv "$tmp/Cupid.app" "$dest/Cupid.app"
  # The build is unsigned; curl never sets the quarantine flag, but strip it
  # anyway so an upgrade over a browser-downloaded copy still opens.
  xattr -dr com.apple.quarantine "$dest/Cupid.app" 2>/dev/null || true

  info "installed $dest/Cupid.app"
  info "open it from Spotlight, or run: open -a Cupid"
}

install_linux() {
  prefix="${CUPID_PREFIX:-$HOME/.local}"
  libdir="$prefix/lib/cupid"
  mkdir -p "$libdir" "$prefix/bin" "$prefix/share/applications" "$prefix/share/pixmaps"

  mv "$tmp/$asset" "$libdir/Cupid.AppImage"
  chmod +x "$libdir/Cupid.AppImage"
  ln -sf "$libdir/Cupid.AppImage" "$prefix/bin/cupid"

  # Pull the largest bundled icon out of the AppImage for the desktop entry.
  icon="application-x-executable"
  if (cd "$tmp" && "$libdir/Cupid.AppImage" --appimage-extract >/dev/null 2>&1); then
    src=$(find "$tmp/squashfs-root" -name '*.png' -exec ls -S {} + 2>/dev/null | head -n1)
    if [ -n "${src:-}" ] && [ -f "$src" ]; then
      cp "$src" "$prefix/share/pixmaps/cupid.png"
      icon="cupid"
    fi
  fi

  cat > "$prefix/share/applications/cupid.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Cupid
Comment=Sheares CCA committee allocation console
Exec=$libdir/Cupid.AppImage %U
Icon=$icon
Terminal=false
Categories=Office;
EOF
  update-desktop-database "$prefix/share/applications" >/dev/null 2>&1 || true

  info "installed $libdir/Cupid.AppImage"
  info "launch it from your app menu, or run: cupid"

  case ":$PATH:" in
    *":$prefix/bin:"*) ;;
    *) info "add $prefix/bin to PATH to run \`cupid\` from a shell" ;;
  esac

  if ! command -v fusermount >/dev/null 2>&1 && ! command -v fusermount3 >/dev/null 2>&1; then
    info "AppImages need FUSE — if Cupid refuses to start, install libfuse2"
  fi
}

case "$os" in
  Darwin) install_macos ;;
  Linux)  install_linux ;;
esac
