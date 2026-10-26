#!/bin/sh
set -eu

repo=QuentinClyy/ocppd
dir=${OCPPD_INSTALL_DIR:-$HOME/.local/bin}
download=${OCPPD_DOWNLOAD:-https://github.com/$repo/releases/latest/download}

case "$(uname -s)-$(uname -m)" in
Linux-x86_64) target=x86_64-unknown-linux-musl ;;
Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-musl ;;
Darwin-arm64) target=aarch64-apple-darwin ;;
Darwin-x86_64) target=x86_64-apple-darwin ;;
*)
	echo "ocppd: no build for $(uname -s) $(uname -m); see https://github.com/$repo#install" >&2
	exit 1
	;;
esac

archive=ocppd-$target.tar.gz
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

curl -fsSL "$download/$archive" -o "$work/$archive"
curl -fsSL "$download/$archive.sha256" -o "$work/$archive.sha256"
if command -v sha256sum >/dev/null; then
	(cd "$work" && sha256sum -c --quiet "$archive.sha256")
else
	(cd "$work" && shasum -a 256 -c --quiet "$archive.sha256")
fi
tar xzf "$work/$archive" -C "$work"

old=$("$dir/ocppd" -V 2>/dev/null || true)
mkdir -p "$dir"
for binary in ocppd ocppd-tui; do
	cp "$work/ocppd-$target/$binary" "$dir/.$binary.new"
	chmod 755 "$dir/.$binary.new"
	mv -f "$dir/.$binary.new" "$dir/$binary"
done
new=$("$dir/ocppd" -V)

if [ -n "$old" ] && [ "$old" != "$new" ]; then
	echo "${old#ocppd } -> ${new#ocppd } in $dir; a running daemon keeps ${old#ocppd } until you press Q in the TUI"
else
	echo "$new in $dir"
fi
case ":$PATH:" in
*":$dir:"*) ;;
*) echo "add $dir to your PATH: export PATH=\"$dir:\$PATH\"" ;;
esac
