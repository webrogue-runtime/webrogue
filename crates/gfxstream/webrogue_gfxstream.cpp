#include <stdlib.h>
#include "gl/gles2_dec/GLESv2Decoder.h"
#include <cstring>

#ifdef min
#undef min
#endif

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
    virtual const unsigned char* readFully(void* buf, size_t len) { 
      printf("WebrogueOutputStream::readFully not implemented\n");
      abort();
    }
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
        memcpy((char*)m_ret_buffer + m_ret_buf_used, buf, len);
        m_ret_buf_used = needed_size;
        return len;
    }

    virtual void* getDmaForReading(uint64_t guest_paddr) { return nullptr; }
    virtual void unlockDma(uint64_t guest_paddr) {}

    virtual void onSave(android::base::Stream* stream) { 
      printf("WebrogueOutputStream::onSave not implemented\n");
      abort();
    }
    virtual unsigned char* onLoad(android::base::Stream* stream) { 
      printf("WebrogueOutputStream::onLoad not implemented\n");
      abort();
    }

    virtual const unsigned char *readRaw(void *buf, size_t *inout_len) { 
      printf("WebrogueOutputStream::readRaw not implemented\n");
      abort();
    }

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
      memcpy((char*)m_input_buffer + m_input_buf_used, buf, len);
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
      return (char*)m_input_buffer + m_input_buf_consumed;
    }

    size_t getIncompleteCommitSize() {
      return m_input_buf_used - m_input_buf_consumed;
    }
};

typedef void *(*get_proc_func_t)(const char *name, void *userData);

class GFXStreamThread {
public:
  std::unique_ptr<gfxstream::gl::GLESv2Decoder> gles2dec;
  std::unique_ptr<ChecksumCalculator> checksum_calculator;
  std::unique_ptr<WebrogueOutputStream> webrogue_output_stream;
  

  GFXStreamThread(get_proc_func_t get_proc, void* userdata) {
    gles2dec = std::make_unique<gfxstream::gl::GLESv2Decoder>();
    checksum_calculator = std::make_unique<ChecksumCalculator>();
    webrogue_output_stream = std::make_unique<WebrogueOutputStream>(16);
    gles2dec->initGL(get_proc, userdata);
  }
};

extern "C" {
void* webrogue_gfxstream_ffi_create_thread(void *get_proc, void* userdata) {
  return new GFXStreamThread((get_proc_func_t) get_proc, userdata);
}

void webrogue_gfxstream_ffi_destroy_thread(void *raw_thread_ptr) {
  GFXStreamThread *thread = (GFXStreamThread *)raw_thread_ptr;
  delete thread;
}

void webrogue_gfxstream_ffi_commit_buffer(void *raw_thread_ptr, void const* buf, uint32_t len) {
  GFXStreamThread *thread = (GFXStreamThread *)raw_thread_ptr;
  WebrogueOutputStream *stream = thread->webrogue_output_stream.get();
  if(stream->getIncompleteCommitSize()) {
    stream->addIncompleteCommit(buf, len);
    size_t decoded = thread->gles2dec->decode(
      stream->getIncompleteCommit(),
      stream->getIncompleteCommitSize(),
      thread->webrogue_output_stream.get(),
      thread->checksum_calculator.get()
    );
    stream->consumeIncompleteCommit(decoded);
  } else {
    size_t decoded = thread->gles2dec->decode(
      (void*)buf,
      len,
      thread->webrogue_output_stream.get(),
      thread->checksum_calculator.get()
    );
    if(decoded<len) {
      stream->addIncompleteCommit((char*)buf + decoded, len-decoded);
    }
  }
}
void webrogue_gfxstream_ffi_ret_buffer_read(void *raw_thread_ptr, void* buf, uint32_t len) {
  GFXStreamThread *thread = (GFXStreamThread *)raw_thread_ptr;
  WebrogueOutputStream *stream = thread->webrogue_output_stream.get();
  size_t available = stream->m_ret_buf_used - stream->m_ret_buf_consumed;
  assert(len<=available);
  size_t to_read = std::min((size_t) len, stream->m_ret_buf_used);
  memcpy(buf, (char*)stream->m_ret_buffer + stream->m_ret_buf_consumed, to_read);
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
