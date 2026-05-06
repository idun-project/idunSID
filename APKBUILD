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
	cd "$srcdir"/idunsnd
  	cargo build --release --locked
}

package() {
  install -D -m 755 "$builddir"/target/release/idunsnd "${pkgdir}"/usr/bin/idunsnd
  install -D -m 644 "$builddir"/idunsnd.rc "${pkgdir}"/etc/init.d/idunsnd
  install -D -m 644 "$builddir"/asound.conf  "${pkgdir}"/etc/asound.conf
}
sha512sums="
6f4dd8aff91aa66a2480f3d1be5f3b02aa771352d688bbe286ad9aee6945221c068660c4c98f3e2ec8c6b362e0909e6b75428a1bf19afd41d013933b8da86c64  idunsnd-1.0.1.tar.gz
"
