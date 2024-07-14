#! /usr/bin/env bash
set -e

CURR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
APP_SRC_DIR="${CURR_DIR}/../user"
APPBINS_DIR="${CURR_DIR}/../appbins"

function main() {
    local target=$1
    local mode=$2

    # make apps
    pushd ${APP_SRC_DIR} > /dev/null
    make build
    popd > /dev/null

    if [ ! -d ${APPBINS_DIR} ]; then
        mkdir ${APPBINS_DIR}
    fi

    for file in ${APP_SRC_DIR}/src/bin/*
    do
        if [ -f "$file" ]; then
            filename=$(basename "$file")
            name_wo_ext="${filename%.*}"
            elf="${APP_SRC_DIR}/target/${target}/${mode}/${name_wo_ext}"
            cp ${elf} ${APPBINS_DIR}
        fi
    done
}

main "$@"