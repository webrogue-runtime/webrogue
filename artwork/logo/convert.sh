cd $(dirname $0)

set -e

# sudo snap install svgo
# sudo apt install librsvg2-bin

alias rpngconvert="convert -define png:exclude-chunks=date,time,gama +set date:create +set date:modify +set date:timestamp "
# Writers
to_png_transparent() {
    rsvg-convert -w $1 -h $1 /dev/stdin -o $2
}
to_png_transparent_wide() {
    rsvg-convert -w $2 -h $2 /dev/stdin -o tmp.png 
    rpngconvert -size $1x$2 xc:none -background none -page +$(expr \( $1 - $2 \) / 2)+0 tmp.png -flatten $3
    rm tmp.png
}
to_png_white() {
    rsvg-convert -w $1 -h $1 --background-color white /dev/stdin -o $2
}
to_svg() {
    svgo /dev/stdin -o $1
}
to_ico() {
    rsvg-convert -w 256 -h 256 /dev/stdin -o tmp.png
    convert -background transparent -define "icon:auto-resize=$2" tmp.png $1
    rm tmp.png
}

# Sizes
ofsize() {
    V=$(expr $(expr $(expr 64 - $1) \* 64) / $1 / 2)
    SIZE=$(expr 64 + $V + $V)
    OFFSET=$(expr 0 - $V)
    cat logo.svg | sed "s/viewBox=\"-1 -1 66 66\"/viewBox=\"$OFFSET $OFFSET $SIZE $SIZE\"/g"
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
    sed "s/stroke:#STROKE_COLOR/stroke:#$1/g"
}
# Stroke width
DEFAULT_STROKE_WIDTH=1.5
stroke_width() {
    sed "s/stroke-width:$DEFAULT_STROKE_WIDTH/stroke-width:$1/g"
}

# For Android
android_old() {
    ofsize 40 | stroke_width 2 | to_png_white $1 ../../android/runtime/launcher/src/main/res/mipmap-$2/ic_launcher.png
}
android_old 48 mdpi
android_old 72 hdpi
android_old 96 xhdpi
android_old 144 xxhdpi
android_old 192 xxxhdpi
ofsize 62 | to_png_transparent 512 ../../android/runtime/launcher/src/main/res/drawable/ic_launcher_foreground.png
ofsize 62 | parse_colors | outer_fill_color 00000000 | default_colors | to_png_transparent 512 ../../android/runtime/launcher/src/main/res/drawable/ic_launcher_monochrome.png
rpngconvert -size 512x512 xc:white ../../android/runtime/launcher/src/main/res/drawable/ic_launcher_background.png
# # For Linux
# ofsize 64 | to_png_transparent 256 ../../platforms/Linux/TemplateAppDir/.DirIcon
# ofsize 64 | to_svg ../../platforms/Linux/TemplateAppDir/webrogue.svg

# # For Web
ofsize 61 | stroke_width 2.5 | to_ico ../../web/root/logo.ico "16,24,32,64"
ofsize 62 | to_png_transparent 1024 ../../web/logo.png
ofsize 62 | to_png_transparent 1024 vscode_extension_logo.png
ofsize 51 | to_png_transparent 128 microsoft_marketplace_logo.png

# # For Windows
# ofsize 64 | margin -6 | to_ico ../../platforms/Windows/logo.ico "16,32,48,256"
# ofsize 64 | margin -5 | to_png_transparent 48 ../../platforms/Windows/Images/LockScreenLogo.png
# ofsize 64 | margin  1 | to_png_transparent_wide 1240 600 ../../platforms/Windows/Images/SplashScreen.png
# ofsize 64 | to_png_transparent 300 ../../platforms/Windows/Images/Square150x150Logo.png
# ofsize 64 | to_png_transparent 88 ../../platforms/Windows/Images/Square44x44Logo.png
# ofsize 64 | to_png_transparent 24 ../../platforms/Windows/Images/Square44x44Logo.targetsize-24_altform-unplated.png
# ofsize 64 | margin -5 | to_png_transparent 50 ../../platforms/Windows/Images/StoreLogo.png
# ofsize 64 | margin  1 | to_png_transparent_wide 620 300 ../../platforms/Windows/Images/Wide310x150Logo.png #620x300

# For XCode
ofsize 44 | to_png_transparent 1024 ../../apple/runtime/AppIcon.icon/Assets/icon_normal.png
ofsize 44 | parse_colors | outer_fill_color 000000 | inner_fill_color ffffff | stroke_color ffffff | default_colors | to_png_transparent 1024 ../../apple/runtime/AppIcon.icon/Assets/icon_dark.png
ofsize 44 | parse_colors | outer_fill_color ffffff80 | inner_fill_color ffffff | stroke_color ffffff | default_colors | to_png_transparent 1024 ../../apple/runtime/AppIcon.icon/Assets/icon_mono.png
