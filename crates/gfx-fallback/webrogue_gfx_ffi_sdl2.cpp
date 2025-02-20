#include "SDL.h"
#include "SDL_video.h"
#include "webrogue_gfx_ffi.h"
#include <stdlib.h>
#include "gl/gles2_dec/GLESv2Decoder.h"
#include "../../external/gfxstream/include/GFXSTREAM_webrogue_unimplemented.h"
#include <cstring>

class WebrogueOutputStream : public gfxstream::IOStream {
  public:
    explicit WebrogueOutputStream(size_t bufsize ):
      gfxstream::IOStream(bufsize) {}
    
    ~WebrogueOutputStream() {};

    void* m_ret_buffer = nullptr;
    size_t m_ret_bufsize = 0;
    size_t m_ret_buf_used = 0;
    size_t m_ret_buf_consumed = 0;

    virtual void* allocBuffer(size_t minSize) {
        size_t allocSize = (m_bufsize < minSize ? minSize : m_bufsize);
        if (!m_buf) {
            m_buf = (unsigned char *)malloc(allocSize);
        }
        else if (m_bufsize < allocSize) {
            unsigned char *p = (unsigned char *)realloc(m_buf, allocSize);
            if (p != NULL) {
                m_buf = p;
                m_bufsize = allocSize;
            } else {
                printf("realloc (%zu) failed\n", allocSize);
                free(m_buf);
                m_buf = NULL;
                m_bufsize = 0;
            }
        }

        return m_buf;
    }
    virtual int commitBuffer(size_t size) { 
        if (size == 0) return 0;
        return writeFully(m_buf, size);
    }
    virtual const unsigned char* readFully(void* buf, size_t len) { GFXSTREAM_NOT_IMPLEMENTED; }
    virtual int writeFully(const void* buf, size_t len) {
        size_t needed_size = m_ret_buf_used + len;
        if(needed_size > m_ret_bufsize) {
          if(m_ret_buffer) {
            m_ret_buffer = realloc(m_ret_buffer, needed_size);
          } else {
            m_ret_buffer = malloc(needed_size);
          }
          m_ret_bufsize = needed_size;
        }
        memcpy(m_ret_buffer + m_ret_buf_used, buf, len);
        m_ret_buf_used = needed_size;
        return len;
    }

    virtual void* getDmaForReading(uint64_t guest_paddr) { return nullptr; }
    virtual void unlockDma(uint64_t guest_paddr) {}

    virtual void onSave(android::base::Stream* stream) { GFXSTREAM_NOT_IMPLEMENTED; }
    virtual unsigned char* onLoad(android::base::Stream* stream) { GFXSTREAM_NOT_IMPLEMENTED; }

    virtual const unsigned char *readRaw(void *buf, size_t *inout_len) { GFXSTREAM_NOT_IMPLEMENTED; }

    // buffer for incomplete commits

    void* m_input_buffer = nullptr;
    size_t m_input_bufsize = 0;
    size_t m_input_buf_used = 0;
    size_t m_input_buf_consumed = 0;

    int addIncompleteCommit(const void* buf, size_t len) {
      size_t needed_size = m_input_buf_used + len;
      if(needed_size > m_input_bufsize) {
        if(m_input_buffer) {
          m_input_buffer = realloc(m_input_buffer, needed_size);
        } else {
          m_input_buffer = malloc(needed_size);
        }
        m_input_bufsize = needed_size;
      }
      memcpy(m_input_buffer + m_input_buf_used, buf, len);
      m_input_buf_used = needed_size;
      return len;
    }

    void consumeIncompleteCommit(size_t len) {
      m_input_buf_consumed += len;
      assert(m_input_buf_consumed<=m_input_buf_used);
      if(m_input_buf_consumed >= m_input_buf_used) {
        m_input_buf_consumed = 0;
        m_input_buf_used = 0;
      }
    }

    void* getIncompleteCommit() {
      return m_input_buffer + m_input_buf_consumed;
    }

    size_t getIncompleteCommitSize() {
      return m_input_buf_used - m_input_buf_consumed;
    }
};

struct System {

};

extern "C" {
void *webrogue_gfx_ffi_create_system(void) {
  System *system_ptr = new System();
  SDL_Init(SDL_INIT_VIDEO);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_ES);
  SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);

  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 2);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);

  return system_ptr;
}
void webrogue_gfx_ffi_destroy_system(void *raw_system_ptr) {
  System *system_ptr = (System *)raw_system_ptr;
  delete system_ptr;
}
static void *get_proc_func(const char *name, void *userData) {
  return SDL_GL_GetProcAddress(name);
}
class Window {
public:
  SDL_Window *sdl_window;
  std::unique_ptr<gfxstream::gl::GLESv2Decoder> gles2dec;
  std::unique_ptr<ChecksumCalculator> checksum_calculator;
  std::unique_ptr<WebrogueOutputStream> webrogue_output_stream;
  

  Window() {
    sdl_window = SDL_CreateWindow(
      "webrogue", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, 800, 450,
      SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE | SDL_WINDOW_ALLOW_HIGHDPI);
    SDL_GL_CreateContext(sdl_window);
    gles2dec = std::make_unique<gfxstream::gl::GLESv2Decoder>();
    checksum_calculator = std::make_unique<ChecksumCalculator>();
    webrogue_output_stream = std::make_unique<WebrogueOutputStream>(16);
    gles2dec->initGL(get_proc_func, nullptr);
  }
};
void *webrogue_gfx_ffi_create_window(void *raw_system_ptr) {
  // System *system_ptr = (System *)raw_system_ptr;
  Window *window_ptr = new Window();
  return window_ptr;
}
void webrogue_gfx_ffi_destroy_window(void *raw_window_ptr) {
  Window *window_ptr = (Window *)raw_window_ptr;
  SDL_DestroyWindow(window_ptr->sdl_window);
  delete window_ptr;
}
void *webrogue_gfx_ffi_gl_get_proc_address(void *raw_system_ptr,
                                           char *procname) {
  return SDL_GL_GetProcAddress(procname);
}
void webrogue_gfx_ffi_get_window_size(void *raw_window_ptr, uint32_t *out_width,
                                      uint32_t *out_height) {
  Window *window_ptr = (Window *)raw_window_ptr;
  int width, height;
  SDL_GetWindowSize(window_ptr->sdl_window, &width, &height);
  *out_width = width;
  *out_height = height;
}
void webrogue_gfx_ffi_get_gl_size(void *raw_window_ptr, uint32_t *out_width,
                                  uint32_t *out_height) {
  Window *window_ptr = (Window *)raw_window_ptr;
  int width, height;
  SDL_GL_GetDrawableSize(window_ptr->sdl_window, &width, &height);
  *out_width = width;
  *out_height = height;
}
void webrogue_gfx_ffi_present_window(void *raw_window_ptr) {
  Window *window = (Window *)raw_window_ptr;
  SDL_GL_SwapWindow(window->sdl_window);
  SDL_Event event;
  SDL_PollEvent(&event);
}
void webrogue_gfx_ffi_gl_init(void *raw_window_ptr) {
  Window *window = (Window *)raw_window_ptr;
}
void webrogue_gfx_ffi_gl_commit_buffer(void *raw_window_ptr, void const* buf, uint32_t len) {
  Window *window = (Window *)raw_window_ptr;
  WebrogueOutputStream *stream = window->webrogue_output_stream.get();
  if(stream->getIncompleteCommitSize()) {
    stream->addIncompleteCommit(buf, len);
    size_t decoded = window->gles2dec->decode(
      stream->getIncompleteCommit(),
      stream->getIncompleteCommitSize(),
      window->webrogue_output_stream.get(),
      window->checksum_calculator.get()
    );
    stream->consumeIncompleteCommit(decoded);
  } else {
    size_t decoded = window->gles2dec->decode(
      (void*)buf,
      len,
      window->webrogue_output_stream.get(),
      window->checksum_calculator.get()
    );
    if(decoded<len) {
      stream->addIncompleteCommit(buf+decoded, len-decoded);
    }
  }
}
void webrogue_gfx_ffi_gl_ret_buffer_read(void *raw_window_ptr, void* buf, uint32_t len) {
  Window *window = (Window *)raw_window_ptr;
  WebrogueOutputStream *stream = window->webrogue_output_stream.get();
  size_t available = stream->m_ret_buf_used - stream->m_ret_buf_consumed;
  assert(len<=available);
  size_t to_read = std::min((size_t)len, stream->m_ret_buf_used);
  memcpy(buf, stream->m_ret_buffer + stream->m_ret_buf_consumed, to_read);
  if(to_read == available) {
    stream->m_ret_buf_used = 0;
    stream->m_ret_buf_consumed = 0;
  } else {
    stream->m_ret_buf_consumed += to_read;
  }
}
}

void gfxstream::gl::gles2_unimplemented() {
  fprintf(stderr, "Called unimplemented GLES API\n");
}
