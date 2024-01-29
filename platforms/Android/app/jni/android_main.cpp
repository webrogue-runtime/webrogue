#include "../../../../src/core/webrogueMain.hpp"
#include "../../../../src/outputs/sdl/SDLOutput.hpp"
#include "SDL.h"
#include "jni.h"

int SDL_main(int argc, char *argv[]) {
    webrogue::core::Config config;
    JNIEnv *jniEnv = (JNIEnv *)SDL_AndroidGetJNIEnv();
    jclass clazz = jniEnv->FindClass("com/webrogue/WebrogueActivity");
    jmethodID get_storage_path_method_id = jniEnv->GetStaticMethodID(
        clazz, "staticGetStoragePath", "()Ljava/lang/String;");
    jstring returned_string = static_cast<jstring>(
        jniEnv->CallStaticObjectMethod(clazz, get_storage_path_method_id));
    config.setDataPath(jniEnv->GetStringUTFChars(returned_string, 0));
    config.loadsModsFromDataPath = true;

    jmethodID get_core_data_method_id =
        jniEnv->GetStaticMethodID(clazz, "staticGetCoreData", "()[B");

    jbyteArray core_array = (jbyteArray)jniEnv->CallStaticObjectMethod(
        clazz, get_core_data_method_id);

    jbyte *core_ptr = jniEnv->GetByteArrayElements(core_array, NULL);
    jsize core_size = jniEnv->GetArrayLength(core_array);
    config.addWrmodData((const uint8_t *)core_ptr, core_size, "core");

    webrogue::core::webrogueMain(
        std::make_shared<webrogue::outputs::sdl::SDLOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
    jniEnv->ReleaseByteArrayElements(core_array, core_ptr, 0);

    return 0;
}
