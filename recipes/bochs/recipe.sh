VERSION=2.7
GIT=https://gitlab.redox-os.org/enygmator/bochs.git
#BRANCH=

#BOCHS source path for local development
# $ROOT is `redox/cookbook`
#BOCHS_PATH=$ROOT/../../redox_apps/bochs/bochs

#Dependencies of this build on other packages available in Redox OS
BUILD_DEPENDS=(zlib mesa liborbital sdl sdl2)

#this function is run if `build` folder doesn't exist
function recipe_prepare {
    #Use default `prepare` function and don't copy source to build
    skip=0
    #and don't copy source to build
    PREPARE_COPY=0
}

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {

    sysroot="$(realpath ../sysroot)"

    export CFLAGS="-I$sysroot/include"
    export CPPFLAGS="-I$sysroot/include"
    export CXXFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"

    # `prefix` and related paths refer to the location of stuff in redoxfs
    # --enable-redox enables --enable-static=yes along with ` -lorbital -lOSMesa -lz` BUT NOT FOR plugins
    # sdl and sdl2 are mutually exclusive
    # es1370 is the sound card - disabled

    ../source/bochs/configure \
        --prefix="/" \
        \
        --build="${BUILD}" \
        --host="${HOST}" \
        --enable-all-optimizations \
        --enable-long-phy-address \
        --enable-cpu-level=6 \
        --enable-x86-64 \
        --enable-vmx=2 \
        --enable-pci \
        --enable-clgd54xx \
        --enable-voodoo \
        --enable-usb \
        --enable-usb-ohci \
        --enable-usb-ehci \
        --enable-usb-xhci \
        --enable-busmouse \
        --enable-show-ips \
        --enable-shared=no \
        --enable-redox \
        --with-nogui \
        --enable-static=yes \
        --with-sdl2 \

    "$REDOX_MAKE" -j"$($NPROC)"

    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" dist-clean
    skip=1
}

function recipe_stage {

    dest="$(realpath $1)"

    "$REDOX_MAKE" DESTDIR="$dest" install

    skip=1
}
