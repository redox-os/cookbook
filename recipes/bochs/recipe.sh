VERSION=2.7
#TAR=

#temporary: source BOCHS source path
BOCHS_PATH=$ROOT/../../redox_apps/bochs/bochs

#Dependencies of this build on other packages available in Redox OS
#BUILD_DEPENDS=(curl glib libiconv libjpeg libpng pcre pixman sdl sdl2 zlib liborbital nghttp2 openssl gettext)
BUILD_DEPENDS=(zlib mesa liborbital sdl sdl2)

#this function is run if source folder doesn't exist
function recipe_fetch {
    #TODO: Temporarily use direct path to BOCHS directory, instead of cloning from git or downloading TAR from personal git

    #temporarily disabling copying, since `configure` uses VPATH (separate build tree)
    mkdir source

    #Since we are copying, we skip the download of TAR
    skip=1
}

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

    $BOCHS_PATH/configure \
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
    #    --with-sdl \
    #    --enable-es1370 \
    #    #--enable-e1000 \
    #    #--enable-ne2000 \


    echo -e "\e[1;33;44m CONFIGURED BOCHS \e[0m"

    "$REDOX_MAKE" -j"$($NPROC)"

    echo -e "\e[1;33;44m MADE BOCHS \e[0m"

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

    echo -e "\e[1;33;44m INSTALLED BOCHS \e[0m"

    skip=1
}
