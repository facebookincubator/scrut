#!/usr/bin/env bash
set -eu
stty -echo -icanon
printf 'hello\033[6n'
IFS= read -r -t 3 -d R reply
test "$reply" = $'\033[1;6'
printf '\r\nName: '
IFS= read -r -t 3 name
printf '\r\nHello %s\r\n' "$name"
