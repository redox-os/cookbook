#!/usr/bin/env bash

set -euo pipefail

declare -r MAIN_BRANCH="master"
declare -r REPO_ROOT="$(git rev-parse --show-toplevel)"
declare -r SCRIPT_DIR="$(dirname "$(realpath "$0")")"

declare -r YELLOW="\e[0;33m"
declare -r GREEN="\e[0;32m"
declare -r BLUE="\e[0;34m"
declare -r RESET="\e[0m"

# Tricky logic: check where we've been ran
if [[ -d "${REPO_ROOT}/cookbook/recipes" && -f "./Makefile" ]] ; then
    # We in the `redox` repo, so where we're supposed to be
    echo -e "${GREEN}Looks like we're launched from right location.">&2
elif [[ -d "${REPO_ROOT}/recipes" ]] ; then
    echo -e "${YELLOW}Running in \`cookbook\` repo, attempt to go up in hierarchy...${RESET}" >&2
    pushd ..
    if [[ -d "./cookbook/recipes" && -f "./Makefile" ]] ; then
        # OK, we're were we supposed to be
        # TODO: is there a better way to do it? Like `make -C`?
        echo -e "${GREEN}Good, \`./cookbook/recipes\` and \`Makefile\` is what we're after.${RESET}" >&2
    fi
fi

pushd cookbook # Go back for a while to scan submodule changes
declare -a affected_recipes
# For each file that current branch and index has changes, try what directories have `recipe.toml`
for changed_file in $(git diff --name-only ${MAIN_BRANCH} .) ; do
    affected_dir=$(dirname "${changed_file}")
    if [[ -f "${affected_dir}/recipe.toml" ]] ; then
        affected_recipe="$(basename ${affected_dir})"
        echo -e "${BLUE}Found affected recipe \`${affected_recipe}\`${RESET}" >&2
        affected_recipes+=(${affected_recipe})
    fi
done
popd

# Now make attempt to build all affected recipes in context of whole `redox`
for recipe in ${affected_recipes[@]} ; do
    echo -e "${YELLOW}Cleanly building \`${recipe}\` recipe:${RESET}" >&2
    make "ucr.${recipe}"
done

# Go back to original location
popd || true
