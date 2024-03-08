cd $(dirname $0)

set -ex

# sudo snap install svgo
# sudo apt install librsvg2-bin

# Reader
original() {
    cat logo.svg
}

# Writers
to_png_transparent() {
    rsvg-convert -w $1 -h $1 /dev/stdin -o $2
}
to_png_white() {
    rsvg-convert -w $1 -h $1 --background-color white /dev/stdin -o $2
}
to_svg() {
    svgo /dev/stdin -o $1
}
to_ico() {
    convert -background transparent -define 'icon:auto-resize=16,24,32,64' /dev/stdin $1
}

# Sizes
bigger() {
    sed "s/viewBox=\"0 0 48 48\"/viewBox=\"6 6 36 36\"/g"
}

# Colors
DEFAULT_STROKE_COLOR=000000
DEFAULT_OUTER_FILL_COLOR=ffffff
DEFAULT_INNER_FILL_COLOR=000000
parse_colors() {
    sed "s/fill:#$DEFAULT_OUTER_FILL_COLOR/fill:#OUTER_FILL_COLOR/g" |
    sed "s/fill:#$DEFAULT_INNER_FILL_COLOR/fill:#INNER_FILL_COLOR/g" |
    sed "s/stroke:#$DEFAULT_STROKE_COLOR/stroke:#STROKE_COLOR/g"
}
default_colors() {
    sed "s/fill:#OUTER_FILL_COLOR/fill:#$DEFAULT_OUTER_FILL_COLOR/g" |
    sed "s/fill:#INNER_FILL_COLOR/fill:#$DEFAULT_INNER_FILL_COLOR/g" |
    sed "s/stroke:#STROKE_COLOR/stroke:#$DEFAULT_STROKE_COLOR/g"
}
outer_fill_color() {
    sed "s/fill:#OUTER_FILL_COLOR/fill:#$1/g"
}
inner_fill_color() {
    sed "s/fill:#INNER_FILL_COLOR/fill:#$1/g"
}
stroke_color() {
    sed "s/fill:#STROKE_COLOR/fill:#$1/g"
}
# Stroke width
DEFAULT_STROKE_WIDTH=1.5
stroke_width() {
    sed "s/stroke-width:$DEFAULT_STROKE_WIDTH/stroke-width:$1/g"
}

# For Android
android_old() {
    original | to_png_white $1 ../../platforms/Android/app/src/main/res/mipmap-$2/ic_launcher.png
}
android_old 48 mdpi
android_old 72 hdpi
android_old 96 xhdpi
android_old 144 xxhdpi
android_old 192 xxxhdpi
original | to_png_transparent 512 ../../platforms/Android/app/src/main/res/drawable/ic_launcher_foreground.png
original | stroke_width 2 | parse_colors | outer_fill_color 00000000 | default_colors | to_png_transparent 512 ../../platforms/Android/app/src/main/res/drawable/ic_launcher_monochrome.png

# For Linux
original | to_png_transparent 256 ../../platforms/Linux/TemplateAppDir/.DirIcon
original | to_svg ../../platforms/Linux/TemplateAppDir/webrogue.svg

# For Web
original | bigger | stroke_width 1 | to_ico ../../platforms/Web/logo.ico
