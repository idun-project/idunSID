pkgname=idunsnd
pkgver=1.0.1
pkgrel=1
pkgdesc="Network SID daemon service for Idun"
arch="aarch64"
url="https://github.com/idun-project/idunSID"
license="GPLv3"
depends="alsa-lib alsa-utils"
makedepends="alsa-lib-dev"
source="$pkgname-$pkgver.tar.gz"
builddir="$srcdir"
options="!check"
install="$pkgname.post-install $pkgname.post-upgrade"

build() {
	# Requires dynamic linking
	export RUSTFLAGS="-C target-feature=-crt-static -C link-arg=-dynamic-linker=/lib/ld-musl-aarch64.so.1"
	cd "$srcdir"/idunsnd
  	cargo build --release --locked
}

package() {
  install -D -m 755 "$builddir"/idunsnd/target/release/idunsnd "${pkgdir}"/usr/bin/idunsnd
  install -D -m 755 "$builddir"/idunsnd.rc "${pkgdir}"/etc/init.d/idunsnd
  install -D -m 644 "$builddir"/asound.conf  "${pkgdir}"/etc/asound.conf
}
sha512sums="
3686c4793998d29c1f58bdb853ed565c85f3c07e38c99ddcb58bc52dfb4acbfdb63f7f9a3cf9931f6db2279c3363736c73a97ca960e5a209e515bc258727a709  idunsnd-1.0.1.tar.gz
"
