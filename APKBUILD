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
088cca07d7884b35cd4c451c00676436402a979217fc832c14fbd62c8d676d1f062205309ea6b567db9b1f9b25216c4258016cf1a5f77519aab570eda53f9a43  idunsnd-1.0.1.tar.gz
"
