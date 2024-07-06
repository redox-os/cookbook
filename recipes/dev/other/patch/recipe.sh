VERSION=2.7.6
TAR=https://ftp.gnu.org/gnu/patch/patch-$VERSION.tar.xz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    export LDFLAGS="-static"
    wget -O build-aux/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    autoreconf
    ./configure --build=${BUILD} --host=${HOST} --prefix=/
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    $HOST-strip $1/bin/*
    rm -rf "$1/"{share,lib}
    skip=1
}
