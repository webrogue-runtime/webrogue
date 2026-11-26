cd "$(dirname "$0")"
set -ex

for FILE in *.kt *.pro
do
    sed -i "s/{{package}}/dev.webrogue.launcher/g" $FILE
    sed -i "s/{{package-unescaped}}/dev.webrogue.launcher/g" $FILE
    sed -i "s/{{library}}/launcher/g" $FILE
    sed -i "s/{{class-extension}}//g" $FILE
    sed -i "s/{{class-init}}//g" $FILE
done

test -f ktfmt.jar || { 
    wget https://github.com/Kotlin/ktfmt/releases/download/v0.65.1/ktfmt-0.65.1-with-dependencies.jar -O ktfmt.jar.part \
        && mv ktfmt.jar.part ktfmt.jar
}
java -jar ktfmt.jar --kotlinlang-style *.kt
