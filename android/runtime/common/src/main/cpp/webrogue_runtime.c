#include "SDL.h"
#include "jni.h"
#include <stddef.h>
#include <stdio.h>

void webrogue_main();

int SDL_main(int argc, char *argv[]) {
  webrogue_main();
  return 0;
}

void webrogue_android_print(char *str, size_t len) {
  JNIEnv *jniEnv = (JNIEnv *)SDL_AndroidGetJNIEnv();
  jobject webrogue_activity = (jobject)SDL_AndroidGetActivity();
  jclass webrogue_activity_class =
      (*jniEnv)->GetObjectClass(jniEnv, webrogue_activity);
  jmethodID print_bytes_method_id = (*jniEnv)->GetMethodID(
      jniEnv, webrogue_activity_class, "printBytes", "([B)V");
  jbyteArray arg = (*jniEnv)->NewByteArray(jniEnv, len);
  jbyte *bytes = (*jniEnv)->GetByteArrayElements(jniEnv, arg, 0);
  memcpy(bytes, str, len);
  (*jniEnv)->SetByteArrayRegion(jniEnv, arg, 0, len, bytes);
  (*jniEnv)->ReleaseByteArrayElements(jniEnv, arg, bytes, 0);
  (*jniEnv)->CallVoidMethodA(jniEnv, webrogue_activity, print_bytes_method_id,
                             (const jvalue *)&arg);
}

const char *webrogue_android_path() {
  JNIEnv *jniEnv = (JNIEnv *)SDL_AndroidGetJNIEnv();
  jobject webrogue_activity = (jobject)SDL_AndroidGetActivity();
  jclass webrogue_activity_class =
      (*jniEnv)->GetObjectClass(jniEnv, webrogue_activity);
  jmethodID get_wrapp_path_method_id =
      (*jniEnv)->GetMethodID(jniEnv, webrogue_activity_class,
                             "getContainerPath", "()Ljava/lang/String;");
  jstring returned_string = (jstring)((*jniEnv)->CallObjectMethod(
      jniEnv, webrogue_activity, get_wrapp_path_method_id));
  // leaks!
  jboolean isCopy = 1;
  return (*jniEnv)->GetStringUTFChars(jniEnv, returned_string, &isCopy);
}
