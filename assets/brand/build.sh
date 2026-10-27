#!/usr/bin/env bash
# Regenerate the raster icon set from the SVG sources.
# Requires ImageMagick with the librsvg delegate (`magick -list format | grep SVG`).
set -euo pipefail
cd "$(dirname "$0")"

render() { # <source.svg> <size> <out.png>
  local density
  density=$(awk -v s="$2" 'BEGIN { printf "%.6f", 96 * s / 1024 }')
  magick -density "$density" -background none "$1" -resize "${2}x${2}" "png/$3"
}

# 16-64 px: small optical size (solid pear).
for size in 16 32 48 64; do
  render logo-small.svg "$size" "icon-$size.png"
done

# 96 px and up: primary mark (sheet seams, citation bracket).
for size in 96 128 256 512 1024; do
  render logo.svg "$size" "icon-$size.png"
done

# Full-bleed square tile for platforms that mask it themselves.
render logo-square.svg 180 apple-touch-icon.png
render logo-square.svg 192 icon-192.png
render logo-square.svg 512 icon-maskable-512.png

# Multi-size favicon: 16 and 32 for tabs, 48 for Windows shortcuts.
magick png/icon-16.png png/icon-32.png png/icon-48.png favicon.ico
