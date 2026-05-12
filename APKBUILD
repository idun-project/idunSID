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
8150f8512efed9bd86746de4f143a6d11ff73a4229cf33947bbaf69f7efdf47c4672a87caeb33b99a49e1ef3c8ff907e4da2421c18d961ca6706e0418b4f4f3e  idunsnd-1.0.1.tar.gz
"
