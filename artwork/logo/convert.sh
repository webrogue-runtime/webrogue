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
    cat logo.svg | sed "s/viewBox=\"0 0 64 64\"/viewBox=\"$OFFSET $OFFSET $SIZE $SIZE\"/g"
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

# # For Windows
# ofsize 64 | margin -6 | to_ico ../../platforms/Windows/logo.ico "16,32,48,256"
# ofsize 64 | margin -5 | to_png_transparent 48 ../../platforms/Windows/Images/LockScreenLogo.png
# ofsize 64 | margin  1 | to_png_transparent_wide 1240 600 ../../platforms/Windows/Images/SplashScreen.png
# ofsize 64 | to_png_transparent 300 ../../platforms/Windows/Images/Square150x150Logo.png
# ofsize 64 | to_png_transparent 88 ../../platforms/Windows/Images/Square44x44Logo.png
# ofsize 64 | to_png_transparent 24 ../../platforms/Windows/Images/Square44x44Logo.targetsize-24_altform-unplated.png
# ofsize 64 | margin -5 | to_png_transparent 50 ../../platforms/Windows/Images/StoreLogo.png
# ofsize 64 | margin  1 | to_png_transparent_wide 620 300 ../../platforms/Windows/Images/Wide310x150Logo.png #620x300

# For MacOS
macos_ico() {
    SIZE=$1
    STROKE_WIDTH=$2
    FILENAME=$3

    MACOS_ICO_DIR=../../apple/runtime/macos/launcher/Assets.xcassets/AppIcon.appiconset
    ofsize 32 | stroke_width $STROKE_WIDTH | to_png_transparent $SIZE $MACOS_ICO_DIR/$FILENAME.foreground.png
    MIN_POS=$(expr 100 '*' $SIZE / 1024)
    MAX_POS=$(expr 924 '*' $SIZE / 1024)
    CORNER_RADIUS=$(expr 184 '*' $SIZE / 1024)
    # rounded rectangle(background)
    rpngconvert -size $SIZE'x'$SIZE xc:transparent -fill white -draw "roundrectangle $MIN_POS,$MIN_POS $MAX_POS,$MAX_POS $CORNER_RADIUS,$CORNER_RADIUS" $MACOS_ICO_DIR/$FILENAME.background.png
    # merge foreground and background
    rpngconvert -background none $MACOS_ICO_DIR/$FILENAME.background.png $MACOS_ICO_DIR/$FILENAME.foreground.png -layers flatten -resize $SIZE'x'$SIZE $MACOS_ICO_DIR/$FILENAME.png
    rm $MACOS_ICO_DIR/$FILENAME.background.png $MACOS_ICO_DIR/$FILENAME.foreground.png
}
macos_document_ico() {
    SIZE=$1
    STROKE_WIDTH=$2
    FILENAME=$3

    MACOS_ICO_DIR=../../apple/runtime/macos/launcher/Assets.xcassets/DocumentIcon.iconset
    ofsize 51 | stroke_width $STROKE_WIDTH | to_png_transparent $SIZE $MACOS_ICO_DIR/$FILENAME.png
}
for flavor in macos_ico macos_document_ico; do
    $flavor 16      3   icon_16x16
    $flavor 32      3   icon_16x16@2x
    $flavor 32      2   icon_32x32
    $flavor 64      2   icon_32x32@2x
    $flavor 128     1.5 icon_128x128
    $flavor 256     1.5 icon_128x128@2x
    $flavor 256     1.5 icon_256x256
    $flavor 512     1.5 icon_256x256@2x
    $flavor 512     1.5 icon_512x512
    $flavor 1024    1.5 icon_512x512@2x
done
# For iOS
ofsize 40 | to_png_white 1024 ../../apple/runtime/ios/launcher/Assets.xcassets/AppIcon.appiconset/ios1024.png
ofsize 48 | to_png_transparent 1024 ../../apple/runtime/ios/launcher/Document.png
