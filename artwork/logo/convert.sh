cd $(dirname $0)

set -ex

# sudo snap install svgo
# sudo apt install librsvg2-bin

original() {
    cat logo.svg
}
bigger() {
    sed "s/viewBox=\"0 0 48 48\"/viewBox=\"6 6 36 36\"/g"
}

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

# For Android
android_old() {
    original | to_png_white $1 ../../platforms/Android/app/src/main/res/mipmap-$2/ic_launcher.png
}
android_old 48 mdpi
android_old 72 hdpi
android_old 96 xhdpi
android_old 144 xxhdpi
android_old 192 xxxhdpi
original | to_png_transparent 512 ../../platforms/Android/app/src/main/res/drawable/ic_launcher_black_foreground.png

# For Linux
original | to_png_transparent 256 ../../platforms/Linux/TemplateAppDir/.DirIcon
original | to_svg ../../platforms/Linux/TemplateAppDir/Webrogue.svg

# For Web
original | bigger | to_ico ../../platforms/Web/logo.ico
