pkgname=idunsid
pkgver=1.0
pkgrel=1
pkgdesc="Network SID daemon service for Idun"
arch=("x86_64" "armv7h")
url="https://github.com/idun-project/idunSID"
license=(GPLv3)
depends=(alsa-lib idun)
makedepends=(clang)
provides=(idun-sid)
install="service.install"

build() {
  cargo build --release
}

package() {
  install -D -m 755 ../target/release/idunsid "${pkgdir}"/usr/bin/idunsid
  install -D -m 644 ../idun-sid.service "${pkgdir}"/etc/systemd/system/idun-sid.service
}
