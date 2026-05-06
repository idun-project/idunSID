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
f9c398ebc8e3aaadcbcfad7047a31ca4599d79111f5a7b3289479797c8afb4598f6fca994ac57024636b100191552871b3ad43d2df797da00a71aed757213e61  idunsnd-1.0.1.tar.gz
"
