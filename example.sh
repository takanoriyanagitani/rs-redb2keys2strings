#!/bin/bash

set -u

wsm="./redb2keys2strings.wasm"

tabname=default_redb_table

iredb="./example.redb"
zredb="./example.redb.zstd"

gpath='/guest.d/example.redb'

geninput() {
	echo trying to create the example file "${iredb}"...

	helper="${HOME}/.cargo/bin/jsonl2redb.wasm"

	test -f "${helper}" || exec env wname="${helper}" sh -c '
    echo helper wasm "${wname}" missing.
    echo "you can install it using cargo(crate: rs-jsonl2redb)."
    exit 1
  '

	runtime=$(which wasmtime)

	test -x "${runtime}" || exec sh -c '
    echo wasi runtime wasmtime missing.
    exit 1
  '

	(
		jq -c -n '["helo", "wrld"]'
		jq -c -n '["hello", "world"]'
	) |
		wasmtime run \
			--dir "${PWD}::/guest.d" \
			--env ENV_TABLE_NAME="${tabname}" \
			--env ENV_OUTPUT_REDB_PATH="${gpath}" \
			"${helper}"
}

unpack() {
	which zstdcat | fgrep -q zstdcat || return
	echo trying to unpack the example db...
	cat "${zredb}" | zstdcat >"${iredb}"
}

cnt=0

test -f "${zredb}" && cnt=$((1 + cnt))
test -f "${iredb}" && cnt=$((2 + cnt))

case $cnt in
0)
	geninput
	;;

1)
	# archived redb found
	unpack
	;;

2)
	# redb already exists
	;;

3)
	# redb already exists
	;;

*) ;;
esac

(
	echo helo
	echo hello
	echo world
	echo hello
	echo hello
	echo hello
	echo hello
) |
	wasmtime run \
		--env ENV_TABLE_NAME="${tabname}" \
		--env ENV_REDB_PATH="${gpath}" \
		--dir "${PWD}::/guest.d" \
		"${wsm}" |
	jq -c
