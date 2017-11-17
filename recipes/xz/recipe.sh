VERSION=5.2.3
TAR=https://github.com/xz-mirror/xz/archive/v$VERSION.tar.gz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    wget -O build-aux/config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    ./autogen.sh
    ./configure --host=${HOST} --prefix=/ --enable-threads=no
    make
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    make clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    rm -rf "$dest/share"
    skip=1
}
