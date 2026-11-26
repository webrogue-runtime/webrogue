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

# 