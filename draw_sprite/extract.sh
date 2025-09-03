#!/bin/bash

for f in /Users/thomas/dev/dune-extract/DUNE_3_7.DAT/DN*.BIN;
do
	echo $f
	../target/release/draw_sprite $f 0 0
	../target/release/draw_sprite $f 0 1
	../target/release/draw_sprite $f 0 2
	../target/release/draw_sprite $f 0 3
done
