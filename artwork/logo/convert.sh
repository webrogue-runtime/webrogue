cd $(dirname $0)

set -ex
# sudo snap install svgo
svgo logo.svg -o logo_optimized.svg
# sudo apt install librsvg2-bin
rsvg-convert -w 1024 -h 1024 logo.svg -o logo_black_transparent_1024.png


android_black_white() {
    rsvg-convert -w $1 -h $1 --background-color white logo.svg -o ../../platforms/Android/app/src/main/res/mipmap-$2/ic_launcher.png
}
android_black_white 48 mdpi
android_black_white 72 hdpi
android_black_white 96 xhdpi
android_black_white 144 xxhdpi
android_black_white 192 xxxhdpi

rsvg-convert -w 512 -h 512 logo.svg -o logo_black_transparent_512.png
mv logo_black_transparent_512.png ../../platforms/Android/app/src/main/res/drawable/ic_launcher_black_foreground.png

convert -background transparent -define 'icon:auto-resize=16,24,32,64' logo.svg logo.ico
