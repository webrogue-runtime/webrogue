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
to_png_transparent_wide() {
    rsvg-convert -w $2 -h $2 /dev/stdin -o tmp.png 
    convert -size $1x$2 xc:none -background none -page +$(expr \( $1 - $2 \) / 2)+0 tmp.png -flatten $3
    rm tmp.png
}
to_png_white() {
    rsvg-convert -w $1 -h $1 --background-color white /dev/stdin -o $2
}
to_svg() {
    svgo /dev/stdin -o $1
}
to_ico() {
    convert -background transparent -define "icon:auto-resize=$2" /dev/stdin $1
}

# Sizes
margin() {
    SIZE=$(expr 48 + $1 + $1)
    OFFSET=$(expr 0 - $1)
    sed "s/viewBox=\"0 0 48 48\"/viewBox=\"$OFFSET $OFFSET $SIZE $SIZE\"/g"
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
DEFAULT_STROKE_WIDTH=1
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
original | parse_colors | outer_fill_color 00000000 | default_colors | to_png_transparent 512 ../../platforms/Android/app/src/main/res/drawable/ic_launcher_monochrome.png

# For Linux
original | to_png_transparent 256 ../../platforms/Linux/TemplateAppDir/.DirIcon
original | to_svg ../../platforms/Linux/TemplateAppDir/webrogue.svg

# For Web
original | margin -6 | to_ico ../../platforms/Web/logo.ico "16,24,32,64"

# For Windows
original | margin -6 | to_ico ../../platforms/Windows/logo.ico "16,32,48,256"
original | margin -5 | to_png_transparent 48 ../../platforms/Windows/Images/LockScreenLogo.png
original | margin  1 | to_png_transparent_wide 1240 600 ../../platforms/Windows/Images/SplashScreen.png
original | to_png_transparent 300 ../../platforms/Windows/Images/Square150x150Logo.png
original | to_png_transparent 88 ../../platforms/Windows/Images/Square44x44Logo.png
original | to_png_transparent 24 ../../platforms/Windows/Images/Square44x44Logo.targetsize-24_altform-unplated.png
original | margin -5 | to_png_transparent 50 ../../platforms/Windows/Images/StoreLogo.png
original | margin  1 | to_png_transparent_wide 620 300 ../../platforms/Windows/Images/Wide310x150Logo.png #620x300

# For MacOS
original | to_png_transparent 16 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos16.png
original | to_png_transparent 32 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos16_x2.png
original | to_png_transparent 32 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos32.png
original | to_png_transparent 64 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos32_x2.png
original | to_png_transparent 128 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos128.png
original | to_png_transparent 256 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos128_x2.png
original | to_png_transparent 256 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos256.png
original | to_png_transparent 512 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos256_x2.png
original | to_png_transparent 512 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos512.png
original | to_png_transparent 1024 ../../platforms/MacOS/Webrogue/Assets.xcassets/AppIcon.appiconset/macos512_x2.png

# For iOS
original | to_png_transparent 1024 ../../platforms/iOS/Webrogue/Assets.xcassets/AppIcon.appiconset/ios1024.png
