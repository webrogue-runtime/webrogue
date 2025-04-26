#include "SDL3/SDL.h"
#include "jni.h"
#include <stddef.h>
#include <stdint.h>

void webrogue_main();

int SDL_main(int argc, char *argv[]) {
  webrogue_main();
  return 0;
}

void webrogue_android_print(char *str, size_t len) {
  JNIEnv *jniEnv = (JNIEnv *)SDL_GetAndroidJNIEnv();
  jobject webrogue_activity = (jobject)SDL_GetAndroidActivity();
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

static const char *webrogue_android_string(const char *method_name) {
  JNIEnv *jniEnv = (JNIEnv *)SDL_GetAndroidJNIEnv();
  jobject webrogue_activity = (jobject)SDL_GetAndroidActivity();
  jclass webrogue_activity_class =
      (*jniEnv)->GetObjectClass(jniEnv, webrogue_activity);
  jmethodID get_wrapp_path_method_id = (*jniEnv)->GetMethodID(
      jniEnv, webrogue_activity_class, method_name, "()Ljava/lang/String;");
  jstring returned_string = (jstring)((*jniEnv)->CallObjectMethod(
      jniEnv, webrogue_activity, get_wrapp_path_method_id));
  // leaks!
  jboolean isCopy = 1;
  return (*jniEnv)->GetStringUTFChars(jniEnv, returned_string, &isCopy);
}

const char *webrogue_android_data_path() {
  return webrogue_android_string("getDataPath");
}

static int64_t webrogue_android_int(const char *method_name) {

  JNIEnv *jniEnv = (JNIEnv *)SDL_GetAndroidJNIEnv();
  jobject webrogue_activity = (jobject)SDL_GetAndroidActivity();
  jclass webrogue_activity_class =
      (*jniEnv)->GetObjectClass(jniEnv, webrogue_activity);
  jmethodID get_wrapp_path_method_id = (*jniEnv)->GetMethodID(
      jniEnv, webrogue_activity_class, method_name, "()J");
  jlong returned_int = ((*jniEnv)->CallLongMethod(jniEnv, webrogue_activity,
                                                  get_wrapp_path_method_id));
  return returned_int;
}

int64_t webrogue_android_container_fd() {
  return webrogue_android_int("getContainerFd");
}
int64_t webrogue_android_container_offset() {
  return webrogue_android_int("getContainerOffset");
}
int64_t webrogue_android_container_size() {
  return webrogue_android_int("getContainerSize");
}
