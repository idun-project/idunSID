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
cd26bbbc35547003789d293dfd9cff1f9a99f695fa43514fb1587c0a29c815a4b1e47c72539be5399bf3137f432cef8e86709636373a798ceb94f2adf6421ddf  idunsnd-1.0.1.tar.gz
"
