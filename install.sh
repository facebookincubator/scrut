#!/bin/sh
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

# ignore `local` checks, because it is accounted for
# shellcheck disable=SC3043

get_local_bin() {
    [ -d "$HOME/bin" ] && echo "$HOME/bin" || echo "$HOME/.local/bin"
}

SCRUT_RELEASE_REPO=${SCRUT_RELEASE_REPO:-ukautz/scrut}
SCRUT_INSTALL_DIRECTORY=${SCRUT_INSTALL_DIRECTORY:-$(get_local_bin)}

_TMP_DIR=$(mktemp -d)

cleanup() {
    test ! -d "$_TMP_DIR" || rm -rf "$_TMP_DIR"
}

trap "cleanup" EXIT

ensure_local() {
    # shellcheck disable=SC2034  # deliberately unused
    local ensure_local
}

ensure_local 2>/dev/null || alias local=typeset

set -euo pipefail

die() {
    echo "$1" >&2
    exit 1
}

info() {
    echo "INFO: $1" >&2
}

warn() {
    [ -t 1 ] && printf "\e[31;1mWARNING: %s\e[0m\n" "$1" >&2 || echo "WARNING: $1" >&2
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

require_cmd() {
    if ! check_cmd "$1"; then
        die "required command '$1' not found"
    fi
}

get_latest_version() {
    curl -fsSL --proto '=https' --tlsv1.2 \
        -H "Accept: application/vnd.github+json" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        https://api.github.com/repos/ukautz/scrut/releases/latest |
        grep '"tag_name"' |
        sed -r 's/^.*: *"(..*)",/\1/'
}


check_install_dir_in_path() {
    local _rc_file
    if ! echo "$PATH" | grep -q "$SCRUT_INSTALL_DIRECTORY"; then
        warn "Installation directory \"$SCRUT_INSTALL_DIRECTORY\" is NOT in \$PATH"
        _rc_file="\$HOME/.$(basename ${SHELL})rc"
        warn "Add it manually to your RC file, for example:"
        warn "  echo 'export PATH=\"\$PATH:$SCRUT_INSTALL_DIRECTORY\"' >> ${_rc_file}"
    fi
}

main() {
    local _os
    local _arch
    local _archive
    local _latest
    local _url

    require_cmd "curl"
    require_cmd "grep"
    require_cmd "mktemp"
    require_cmd "sed"
    require_cmd "uname"

    _os=$(uname -s)
    _arch=$(uname -m)

    case "$_os" in
        Darwin)
            _os="macos"
            ;;
        Linux)
            _os="linux"
            ;;
        *)
            die "Unsupported OS '$_os'"
            ;;
    esac

    case "$_arch" in
        aarch64 | arm64)
            _arch="aarch64"
            ;;
        x86_64 | x86-64 | x64 | amd64)
            _arch="x86_64"
            ;;
        *)
            die "Unsupported architecture '$_arch'"
            ;;
    esac

    info "Detected OS/architecture: $_os/$_arch"

    _latest=$(get_latest_version)
    info "Latest Scrut version: $_latest"

    _url="https://github.com/${SCRUT_RELEASE_REPO}/releases/download/${_latest}/scrut-${_latest}-${_os}-${_arch}.tar.gz"
    info "Downloading from: $_url"
    curl -fsSL --proto '=https' --tlsv1.2 -o "$_TMP_DIR/archive.tar.gz" "$_url"

    info "Unpack archive"
    tar -zxf "$_TMP_DIR/archive.tar.gz" -C "$_TMP_DIR"

    info "Installing binary into: $SCRUT_INSTALL_DIRECTORY"
    mkdir -p "$SCRUT_INSTALL_DIRECTORY"
    cp -a "${_TMP_DIR}/scrut-${_os}-${_arch}/scrut" "$SCRUT_INSTALL_DIRECTORY"

    check_install_dir_in_path
}

usage() {
    cat << EOF
$0 [OPTIONS]

Options:
    --help, -h
        Show this help

    --verbose, -v
        Show verbosely everything that is executed

    --owner-repo, -r
        Github owner and repository in format OWNER/REPO (default: $SCRUT_RELEASE_REPO)

    --installation-path, -p
        Set installation path (default: $SCRUT_INSTALL_DIRECTORY)
EOF
}

parse_args() {
    while [ $# -gt 0 ]; do
        case $1 in
            --help | -h)
                usage
                exit 0
                ;;
            --verbose | -v)
                set -x
                shift
                ;;
            --installation-path | -p)
                SCRUT_INSTALL_DIRECTORY="$2"
                shift 2
                ;;
            --owner-repo | -r)
                SCRUT_RELEASE_REPO="$2"
                shift 2
                ;;
            *)
                die "Unknown option: $1"
                ;;
        esac
    done
}

parse_args "$@"
main
