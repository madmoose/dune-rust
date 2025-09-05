#!/bin/bash
set -e
cargo run --bin draw_night_attack --release
sha1sum ppm/* > night-attack-checksums-recreation.txt
cmp night-attack-checksums-correct.txt night-attack-checksums-recreation.txt && echo "ok"
