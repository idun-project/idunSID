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
  cargo build --release --locked
}

package() {
  install -D -m 755 "$builddir"/target/release/idunsnd "${pkgdir}"/usr/bin/idunsnd
  install -D -m 644 "$builddir"/idunsnd.rc "${pkgdir}"/etc/init.d/idunsnd
  install -D -m 644 "$builddir"/asound.conf  "${pkgdir}"/etc/asound.conf
}
sha512sums="
e0eb5be814895f38386968a83321878a818ceefcbc8362144f6dbbad80769cffc4a7de0b19518a21f4212dd3da6962a7fe4e059ae8dbdcde41eabe08f9138dbd  idunsnd-1.0.1.tar.gz
"
