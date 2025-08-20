#!/usr/bin/env bash

set -euo pipefail

declare -r MAIN_BRANCH="master"
declare -r REPO_ROOT="$(git rev-parse --show-toplevel)"
declare -r SCRIPT_DIR="$(dirname "$(realpath "$0")")"

declare -rA TERMCOLORS=(
    ["green"]="\e[0;32m"
    ["blue"]="\e[0;34m"
    ["yellow"]="\e[0;33m"
    ["reset"]="\e[0m"
)

# Print to stderr in colour
function ceprint() {
    declare -r colour="$1"
    declare -r text="$2" # TODO: digest remaining arguments
    echo -e "${TERMCOLORS[${COLOUR}]}${text}${TERMCOLORS["reset"]}" >&2
}

# Tricky logic: check where we've been ran
if [[ -d "${REPO_ROOT}/cookbook/recipes" && -f "./Makefile" ]] ; then
    # We in the `redox` repo, so where we're supposed to be
    ceprint "green" "Looks like we're launched from right location."
elif [[ -d "${REPO_ROOT}/recipes" ]] ; then
    ceprint "yellow" "Running in \`cookbook\` repo, attempt to go up in hierarchy..."
    pushd ..
    if [[ -d "./cookbook/recipes" && -f "./Makefile" ]] ; then
        # OK, we're were we supposed to be
        # TODO: is there a better way to do it? Like `make -C`?
        ceprint "green" "Good, \`./cookbook/recipes\` and \`Makefile\` is what we're after."
    fi
fi

pushd cookbook # Go back for a while to scan submodule changes
declare -a affected_recipes
# For each file that current branch and index has changes, try what directories have `recipe.toml`
for changed_file in $(git diff --name-only ${MAIN_BRANCH} .) ; do
    affected_dir=$(dirname "${changed_file}")
    if [[ -f "${affected_dir}/recipe.toml" ]] ; then
        affected_recipe="$(basename ${affected_dir})"
        ceprint "blue" "Found affected recipe \`${affected_recipe}\`"
        affected_recipes+=(${affected_recipe})
    fi
done
popd

# Now make attempt to build all affected recipes in context of whole `redox`
for recipe in ${affected_recipes[@]} ; do
    ceprint "yellow" "Cleanly building \`${recipe}\` recipe:"
    make "ucr.${recipe}"
done

# Go back to original location
popd || true
