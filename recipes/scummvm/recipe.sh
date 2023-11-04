VERSION=2.7.0
TAR=https://downloads.scummvm.org/frs/scummvm/$VERSION/scummvm-$VERSION.tar.xz
TAR_SHA256=444b1ffd61774fe867824e57bb3033c9998ffa8a4ed3a13246b01611d5cf9993
BUILD_DEPENDS=(sdl liborbital freetype zlib libpng)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-static"
    ./configure \
        --host=${HOST} \
        --prefix='' \
        --with-sdl-prefix="$sysroot" \
        --with-freetype2-prefix="$sysroot" \
        --with-png-prefix="$sysroot" \
        --with-zlib-prefix="$sysroot" \
        --disable-timidity \
        --disable-mt32emu
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

    mkdir -pv "$1/ui/apps"
    cp -v "${COOKBOOK_RECIPE}/manifest" "$1/ui/apps/scummvm"

    mkdir -pv "$1/ui/icons/apps"
    cp -v "${COOKBOOK_RECIPE}/icon.png" "$1/ui/icons/apps/scummvm.png"

    skip=1
}
