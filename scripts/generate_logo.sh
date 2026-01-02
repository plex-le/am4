#!/bin/bash

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
IMG_DIR="$SCRIPT_DIR/../am4/assets/img/icons"
SVGS=("logo-maskable.svg" "logo-monochrome.svg" "logo.svg")
SIZES=(512 196 32)

for svg in "${SVGS[@]}"; do
    filename=$(basename "$svg" .svg)
    for size in "${SIZES[@]}"; do
        output="$IMG_DIR/${filename}-${size}.png"
        echo "generating $output..."
        inkscape -w "$size" -h "$size" "$IMG_DIR/$svg" -o "$output"
    done
done