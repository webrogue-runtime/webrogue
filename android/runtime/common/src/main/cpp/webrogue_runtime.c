#include "SDL3/SDL.h"
#include "jni.h"
#include <stddef.h>
#include <stdint.h>
#include <unistd.h>
#include <pthread.h>
#include <android/log.h>

void webrogue_main();

#ifndef NDEBUG
static int pfd[2] = {0, 0};
static pthread_t thr;
static const char *tag = "webrogue stdout";

static void *thread_func(void* data) {
  ssize_t rdsz;
  char buf[128];
  while((rdsz = read(pfd[0], buf, sizeof buf - 1)) > 0) {
    if(buf[rdsz - 1] == '\n') --rdsz;
    buf[rdsz] = 0;  /* add null-terminator */
    __android_log_write(ANDROID_LOG_DEBUG, tag, buf);
  }
  return 0;
}

static int start_logger() {
  if(pfd[0] || pfd[1]) {
    return 0;
  }
  /* make stdout line-buffered and stderr unbuffered */
  setvbuf(stdout, 0, _IOLBF, 0);
  setvbuf(stderr, 0, _IONBF, 0);

  /* create the pipe and redirect stdout and stderr */
  pipe(pfd);
  dup2(pfd[1], 1);
  dup2(pfd[1], 2);

  /* spawn the logging thread */
  if(pthread_create(&thr, 0, thread_func, 0) == -1)
    return -1;
  pthread_detach(thr);
  return 0;
}
#endif

int SDL_main(int argc, char *argv[]) {
#ifndef NDEBUG
  start_logger();
#endif
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
