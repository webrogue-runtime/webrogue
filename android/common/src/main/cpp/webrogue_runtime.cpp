#include "SDL.h"
#include "jni.h"
#include <cstdio>
#include <stddef.h>

extern "C" void webrogue_main();

extern "C" int SDL_main(int argc, char *argv[]) {
  webrogue_main();
  return 0;
}

extern "C" void webrogue_android_print(char *str, size_t len) {
  auto *jniEnv = (JNIEnv *)SDL_AndroidGetJNIEnv();
  auto *webrogue_activity = (jobject)SDL_AndroidGetActivity();
  jclass webrogue_activity_class = jniEnv->GetObjectClass(webrogue_activity);
  jmethodID print_bytes_method_id =
      jniEnv->GetStaticMethodID(webrogue_activity_class, "printBytes", "([B)V");
  jbyteArray arg = jniEnv->NewByteArray(len);
  jbyte *bytes = jniEnv->GetByteArrayElements(arg, 0);
  memcpy(bytes, str, len);
  jniEnv->SetByteArrayRegion(arg, 0, len, bytes);
  jniEnv->ReleaseByteArrayElements(arg, bytes, 0);
  jniEnv->CallStaticVoidMethodA(webrogue_activity_class, print_bytes_method_id,
                                (const jvalue *)&arg);
}

extern "C" const char *webrogue_android_path() {
  auto *jniEnv = (JNIEnv *)SDL_AndroidGetJNIEnv();
  auto *webrogue_activity = (jobject)SDL_AndroidGetActivity();
  jclass webrogue_activity_class = jniEnv->GetObjectClass(webrogue_activity);
  jmethodID get_wrapp_path_method_id = jniEnv->GetStaticMethodID(
      webrogue_activity_class, "getContainerPath", "()Ljava/lang/String;");
  auto returned_string = static_cast<jstring>(jniEnv->CallStaticObjectMethod(
      webrogue_activity_class, get_wrapp_path_method_id));
  // leaks!
  jboolean isCopy = 1;
  return jniEnv->GetStringUTFChars(returned_string, &isCopy);
}
