#!/usr/bin/env bash
# render-logo.sh: zett's logo from logo.glsl (GPU, ffmpeg's libplacebo) at 1024 px, cut into the rounded tile;
# zett.svg carries the render (a fluid has no clean vector shapes), install.sh makes the sizes from it
set -euo pipefail
cd "$(dirname "$0")"
t=$(mktemp -d); trap 'rm -rf "$t"' EXIT
ffmpeg -hide_banner -loglevel error -y -f lavfi -i color=black:s=1024x1024 -vf "libplacebo=w=1024:h=1024:custom_shader_path=logo.glsl,format=rgb24" -frames:v 1 "$t/raw.png"
magick -size 1024x1024 xc:none -fill white -draw "roundrectangle 64,64 960,960 240,240" "$t/mask.png"
magick "$t/raw.png" "$t/mask.png" -alpha off -compose CopyOpacity -composite -resize 512x512 zett.png
printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><image width="512" height="512" href="data:image/png;base64,%s"/></svg>\n' "$(base64 -w0 zett.png)" > zett.svg
echo "zett.png and zett.svg from logo.glsl"
