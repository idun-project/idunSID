pkgname=idunsid
pkgver=1.0
pkgrel=1
pkgdesc="Network SID daemon service for Idun"
arch=("x86_64" "armv7h")
url="https://github.com/idun-project/idunSID"
license=(GPLv3)
depends=(alsa-lib)
makedepends=(clang)
provides=(idun-sid)
install="service.install"

build() {
  export PKG_CONFIG_SYSROOT_DIR=/usr/arm-linux-gnueabihf
  cargo build --release --target arm-unknown-linux-gnueabihf
}

package() {
  install -D -m 755 ../target/arm-unknown-linux-gnueabihf/release/idunsid "${pkgdir}"/usr/bin/idunsid
  install -D -m 644 ../idun-sid.service "${pkgdir}"/etc/systemd/system/idun-sid.service
}
