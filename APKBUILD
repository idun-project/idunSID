pkgname=idunsnd
pkgver=1.0.1
pkgrel=0
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
a418c57812e3172c4ccff027b4f905eebedecdb4ad25d05b54f73649e91e6e5d1556079e5454a06a302e215b63a4cd072857a5ea43244614ff7afa3d668a8efb  idunsnd-1.0.1.tar.gz
"
