#!/bin/sh
files=$( (\
		find ../assets -type f ; \
		find assets -type f ; \
		find ../**/src -type f | grep .rs \
	) | sort)

hash=$(shasum $files | shasum | cut -b -8)

echo Source hash is $hash

rm -rf output
mkdir output
sed "s/\$hash/$hash/g" < assets/index.html > output/index.html
wasm-pack build --release --out-dir output/pkg --out-name wasm_dune_lipsync_$hash --target web
