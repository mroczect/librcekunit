#!/usr/bin/env bash
# Rilis C binding saja. Pakai: ./scripts/release-c-binding.sh v0.1.0
set -euo pipefail

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  echo "usage: $0 <version>   (contoh: $0 v0.1.0)"
  exit 2
fi

VERSION="${VERSION#v}"   # strip leading v
TARGET="x86_64-unknown-linux-gnu"
PKG="librcekunit-c-binding-v${VERSION}-${TARGET}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> Build C binding"
cargo build --release -p librcekunit_c_binding

echo "==> Package"
rm -rf "dist" "$PKG"
mkdir -p "dist/$PKG/lib" "dist/$PKG/include"

cp target/release/liblibrcekunit.so "dist/$PKG/lib/"
cp target/release/liblibrcekunit.a  "dist/$PKG/lib/" 2>/dev/null || true
cp librcekunit_c_binding/include/librcekunit.h "dist/$PKG/include/"

cat > "dist/$PKG/lib/librcekunit.pc" <<EOF
prefix=\${pcfiledir}/../..
exec_prefix=\${prefix}
libdir=\${prefix}/lib
includedir=\${prefix}/include

Name: librcekunit
Description: C ABI bindings for librcekunit
Version: ${VERSION}
Libs: -L\${libdir} -llibrcekunit -lpthread -ldl
Cflags: -I\${includedir}
EOF

tar -C dist -czf "dist/${PKG}.tar.gz" "$PKG"

echo "==> Checksum"
sha256sum "dist/${PKG}.tar.gz" > "dist/${PKG}.tar.gz.sha256"

echo "==> Tag"
git tag -f "c-binding-v${VERSION}" 2>/dev/null || git tag "c-binding-v${VERSION}"
git push origin "c-binding-v${VERSION}"

echo "==> Done"
ls -lh dist/
echo
echo "Upload tarball ke release page:"
echo "  dist/${PKG}.tar.gz"
echo "  dist/${PKG}.tar.gz.sha256"
