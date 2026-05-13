# Maintainer: Jason Milkins <adventurejason@gmail.com>
pkgname=active-window
pkgver=0.1.0
pkgrel=1
pkgdesc="CLI tool that prints the currently active Wayland window title to stdout"
arch=('x86_64')
url="https://github.com/adventurejason-code/active-window"
license=('MIT')
depends=('gcc-libs')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/adventurejason-code/active-window/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname"
    # Use CARGO_HOME inside srcdir to avoid polluting ~/.cargo during build
    export CARGO_HOME="$srcdir/.cargo"
    cargo build --release
}

package() {
    cd "$pkgname"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    # Install a basic man page if you create one, e.g.:
    # install -Dm644 "man/$pkgname.1" "$pkgdir/usr/share/man/man1/$pkgname.1"
}
