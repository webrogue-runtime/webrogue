CMDLINE_TOOLS_VERSION=linux-11076708_latest
NDK_VERSION=27.3.13750724
ANDROID_API_VERSION=28

if test -z $ANDROID_SDK_ROOT
then
    export ANDROID_SDK_ROOT="$(pwd)/sdk"
    test -d $ANDROID_SDK_ROOT || mkdir $ANDROID_SDK_ROOT
fi
test -d $ANDROID_SDK_ROOT/cmdline-tools || {
    wget https://dl.google.com/android/repository/commandlinetools-$CMDLINE_TOOLS_VERSION.zip -O $ANDROID_SDK_ROOT/commandlinetools-$CMDLINE_TOOLS_VERSION.zip
    unzip $ANDROID_SDK_ROOT/commandlinetools-$CMDLINE_TOOLS_VERSION.zip -d $ANDROID_SDK_ROOT
    rm $ANDROID_SDK_ROOT/commandlinetools-$CMDLINE_TOOLS_VERSION.zip
}

export ANDROID_NDK_PATH="$ANDROID_SDK_ROOT/ndk/$NDK_VERSION"
test -d "$ANDROID_SDK_ROOT/licenses" || yes | $ANDROID_SDK_ROOT/cmdline-tools/bin/sdkmanager --licenses --sdk_root=$ANDROID_SDK_ROOT
test -d "$ANDROID_NDK_PATH" || $ANDROID_SDK_ROOT/cmdline-tools/bin/sdkmanager --sdk_root=$ANDROID_SDK_ROOT "ndk;$NDK_VERSION"
