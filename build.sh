#!/usr/bin/env bash
set -euo pipefail

main() {
    ZOLA_VERSION=0.23.4
    ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    ZOLA="${ROOT_DIR}/zola"
    DEV_ROOT="$(mktemp -d)"

    cleanup() {
        rm -rf "${DEV_ROOT}"
    }
    trap cleanup EXIT

    zola_is_pinned() {
        [[ -x "$1" ]] && [[ "$("$1" --version 2>/dev/null | awk 'NR == 1 { print $2 }')" == "${ZOLA_VERSION}" ]]
    }

    if ! zola_is_pinned "${ZOLA}" && command -v zola >/dev/null 2>&1 && zola_is_pinned "$(command -v zola)"; then
        ZOLA="$(command -v zola)"
    fi

    if ! zola_is_pinned "${ZOLA}"; then
        archive="$(mktemp)"
        curl -sL "https://github.com/getzola/zola/releases/download/v${ZOLA_VERSION}/zola-v${ZOLA_VERSION}-x86_64-unknown-linux-gnu.tar.gz" -o "${archive}"
        tar -xzf "${archive}" -C "${ROOT_DIR}"
        rm -f "${archive}"
    fi

    rm -rf "${ROOT_DIR}/public"
    "${ZOLA}" --root "${ROOT_DIR}" build

    for version_dir in "${ROOT_DIR}"/versions/*; do
        [[ -d "${version_dir}" ]] || continue
        version_slug="$(basename "${version_dir}")"
        version="${version_slug#v}"
        archive_root="$(mktemp -d)"
        mkdir -p "${archive_root}/content"
        cp -a "${version_dir}/content/." "${archive_root}/content/"
        cp "${ROOT_DIR}/config.toml" "${archive_root}/config.toml"
        cp -a "${ROOT_DIR}/templates" "${ROOT_DIR}/sass" "${ROOT_DIR}/static" "${archive_root}/"

        sed -i 's|^base_url = .*|base_url = "https://oximo.dev/'"${version_slug}"'/"|' "${archive_root}/config.toml"
        sed -i 's|^channel = .*|channel = "archive"|' "${archive_root}/config.toml"
        sed -i 's|^current = .*|current = "'"${version}"'"|' "${archive_root}/config.toml"

        "${ZOLA}" --root "${archive_root}" build --base-url "https://oximo.dev/${version_slug}/"
        mkdir -p "${ROOT_DIR}/public/${version_slug}"
        cp -a "${archive_root}/public/." "${ROOT_DIR}/public/${version_slug}/"
        rm -rf "${archive_root}"
    done

    mkdir -p "${DEV_ROOT}/content"
    cp -a "${ROOT_DIR}/content/." "${DEV_ROOT}/content/"
    rm -f "${DEV_ROOT}/content/roadmap.md"
    if [[ -d "${ROOT_DIR}/dev" ]]; then
        cp -a "${ROOT_DIR}/dev/." "${DEV_ROOT}/content/"
    fi
    cp "${ROOT_DIR}/config.toml" "${DEV_ROOT}/config.toml"
    cp -a "${ROOT_DIR}/templates" "${ROOT_DIR}/sass" "${ROOT_DIR}/static" "${DEV_ROOT}/"

    sed -i 's|^base_url = .*|base_url = "https://oximo.dev/dev/"|' "${DEV_ROOT}/config.toml"
    sed -i 's|^channel = .*|channel = "dev"|' "${DEV_ROOT}/config.toml"
    sed -i 's|^current = .*|current = "dev"|' "${DEV_ROOT}/config.toml"

    "${ZOLA}" --root "${DEV_ROOT}" build --base-url "https://oximo.dev/dev/"
    mkdir -p "${ROOT_DIR}/public/dev"
    cp -a "${DEV_ROOT}/public/." "${ROOT_DIR}/public/dev/"
}

main
