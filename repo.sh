#!/usr/bin/env bash
set -e
shopt -s nullglob

source config.sh

APPSTREAM="0"
COOK_OPT=""
recipes=""
for arg in "${@:1}"
do
    if [ "$arg" == "--appstream" ]
    then
        APPSTREAM="1"
    elif [ "$arg" == "--debug" ]
    then
        DEBUG=--debug
    elif [ "$arg" == "--with-package-deps" ]
    then
        COOK_OPT=--with-package-deps
    elif [ "$arg" == "--nonstop" ]
    then
        set +e
    elif [ "$arg" == "--offline" ]
    then
        export COOKBOOK_OFFLINE="1"
    else
        recipes+=" $arg"
    fi
done

if [ "$recipes" == "" ]
then
    recipes="$(target/release/list_recipes)"
fi

# All $recipes that are in the new TOML format.
toml_recipes=""

for recipe in $recipes
do
    recipe_path=`target/release/find_recipe $recipe`
    COOKBOOK_RECIPE="$recipe_path"
    TARGET_DIR="${COOKBOOK_RECIPE}/target/${TARGET}"
    COOKBOOK_BUILD="${TARGET_DIR}/build"
    COOKBOOK_STAGE="${TARGET_DIR}/stage"
    COOKBOOK_SOURCE="${COOKBOOK_RECIPE}/source"
    COOKBOOK_SYSROOT="${TARGET_DIR}/sysroot"

    if [ -e "${COOKBOOK_RECIPE}/recipe.toml" ]
    then
        toml_recipes+=" $recipe"
        target/release/cook $COOK_OPT "$recipe"
        continue
    fi

    if [ ! -d "${COOKBOOK_SOURCE}" ]
    then
        echo -e "\033[01;38;5;155mrepo - fetching $recipe\033[0m" >&2
        ./cook.sh "$recipe" fetch
    fi

    if [ ! -d "${COOKBOOK_BUILD}" ]
    then
        echo -e "\033[01;38;5;155mrepo - preparing $recipe\033[0m" >&2
        ./cook.sh "$recipe" prepare
    elif [ ! -d "${COOKBOOK_SYSROOT}" ]
    then
        echo -e "\033[01;38;5;155mrepo - repreparing $recipe\033[0m" >&2
        ./cook.sh "$recipe" unprepare prepare
    else
        TIME_SOURCE="$($FIND "${COOKBOOK_SOURCE}" -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_BUILD="$($FIND "${COOKBOOK_BUILD}" -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        if [ "$TIME_SOURCE" -gt "$TIME_BUILD" ]
        then
            echo -e "\033[01;38;5;155mrepo - repreparing $recipe\033[0m" >&2
            ./cook.sh "$recipe" unprepare prepare
        fi
    fi

    if [ ! -f "${COOKBOOK_STAGE}.pkgar" ]
    then
        echo -e "\033[01;38;5;155mrepo - building $recipe\033[0m" >&2
        ./cook.sh "$recipe" build stage pkg $DEBUG
    else
        TIME_BUILD="$($FIND "${COOKBOOK_BUILD}" -type f -not -path '*/.git*' -printf "%Ts\n" | sort -nr | head -n 1)"
        TIME_STAGE="$($STAT -c "%Y" "${COOKBOOK_STAGE}.pkgar")"
        TIME_RECIPE="$($FIND "${COOKBOOK_RECIPE}"/{recipe.sh,*.patch} -printf '%Ts\n' | sort -nr | head -n 1)"
        if [ "$TIME_BUILD" -gt "$TIME_STAGE" -o "$TIME_RECIPE" -gt "$TIME_STAGE" ]
        then
            echo -e "\033[01;38;5;155mrepo - rebuilding $recipe\033[0m" >&2
            ./cook.sh "$recipe" untar unstage build stage pkg $DEBUG
        else
            echo -e "\033[01;38;5;155mrepo - $recipe up to date\033[0m" >&2
        fi
    fi
done

mkdir -p "$REPO"

declare -A APPSTREAM_SOURCES

# Currently, we only support runtime dependencies for recipes in the new TOML
# format. Runtime dependencies include both `[package.dependencies]` and
# dynamically linked packages discovered by auto_deps.
# 
# The following adds the package dependencies of the recipes to the repo as
# well.
recipes="$recipes $(target/release/pkg_deps $toml_recipes)"

REPO_BUILDER="./target/release/repo_builder"

if [ -x "$REPO_BUILDER" ] # TODO: Wait until everyone has this binary
then
    "$REPO_BUILDER" "$REPO" $recipes
else # TODO: Delete this soon

for recipe in $recipes
do
    recipe_path=`target/release/find_recipe $recipe`
    COOKBOOK_RECIPE="$recipe_path"
    TARGET_DIR="${COOKBOOK_RECIPE}/target/${TARGET}"
    COOKBOOK_STAGE="${TARGET_DIR}/stage"

    if [ "${COOKBOOK_STAGE}.pkgar" -nt "$REPO/$recipe.pkgar" ]
    then
        echo -e "\033[01;38;5;155mrepo - publishing $recipe\033[0m" >&2
        cp -v "${COOKBOOK_STAGE}.pkgar" "$REPO/$recipe.pkgar"
        cp -v "${COOKBOOK_STAGE}.toml" "$REPO/$recipe.toml"
    fi

    if [ -e "${COOKBOOK_STAGE}/usr/share/metainfo" ]
    then
        APPSTREAM_SOURCES["$recipe"]="${COOKBOOK_STAGE}"
    fi
done

if [ "${APPSTREAM}" == "1" ]
then
    echo -e "\033[01;38;5;155mrepo - generating appstream data\033[0m" >&2

    APPSTREAM_ROOT="$ROOT/build/${TARGET}/appstream"
    APPSTREAM_PKG="$REPO/appstream.pkgar"
    rm -rf "${APPSTREAM_ROOT}" "${APPSTREAM_PKG}"
    mkdir -p "${APPSTREAM_ROOT}"
    if [ "${#APPSTREAM_SOURCES[@]}" -ne 0 ]
    then
        appstreamcli compose \
            --origin=pkgar \
            --result-root="${APPSTREAM_ROOT}" \
            "${APPSTREAM_SOURCES[@]}"
    fi
    pkgar create \
        --archive "${APPSTREAM_PKG}" \
        --skey "${ROOT}/build/id_ed25519.toml" \
        "${APPSTREAM_ROOT}"
fi

echo -e "\033[01;38;5;155mrepo - generating repo.toml\033[0m" >&2

echo "[packages]" > "$REPO/repo.toml"
for toml in "$REPO/"*".toml"
do
    package="$(basename "$toml" .toml)"
    if [ "$package" != "repo" ]
    then
        version="$(grep version "$toml" | cut -d '=' -f2-)"
        echo "$package =$version" >> "$REPO/repo.toml"
    fi
done

fi
