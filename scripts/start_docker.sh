#! /usr/bin/env bash

IMAGE_NAME="croakexciting/ffos_dev_env"
IMAGE_VERSION="0.0.1"
CONTAINER_NAME="ffos_dev"
CURR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
FFOS_ROOT_DIR="$CURR_DIR/../"

function remove_container_if_exists() {
    local container="$1"
    if docker ps -a --format '{{.Names}}' | grep -q "${container}"; then
        echo "Removing existing ffos container: ${container}"
        docker stop "${container}" >/dev/null
        docker rm -v -f "${container}" 2>/dev/null
    fi
}

function main() {
    remove_container_if_exists ${CONTAINER_NAME}

    docker run -itd \
        --name "${CONTAINER_NAME}" \
        -v ${FFOS_ROOT_DIR}:/ffos \
        -w /ffos \
        ${IMAGE_NAME}:${IMAGE_VERSION} \
        /bin/bash
}

main "$@"
