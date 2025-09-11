#include <memory>
#include <stdlib.h>
#include "aemu/base/Metrics.h"
#include "gfxstream/host/BackendCallbacks.h"
#include "ProcessResources.h"
#include "utils/GfxApiLogger.h"
#include "render-utils/IOStream.h"
#include "VkDecoderContext.h"
#include "VkDecoder.h"
#include "snapshot/LazySnapshotObj.h"
#include "RenderThreadInfoVk.h"
#include "goldfish_vk_dispatch.h"
#include "Features.h"
#include "VkCommonOperations.h"
#include "VkDecoderGlobalState.h"
#include "VulkanBoxedHandles.h"
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
        return 0;
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

class MetricsLoggerImpl: public android::base::MetricsLogger {
  void logMetricEvent(android::base::MetricEventType eventType) override {}
  void setCrashAnnotation(const char* key, const char* value) override {}
};

static get_proc_func_t sVkGetProc = nullptr;
static void* sVkGetProcUserdata = nullptr;
static std::unique_ptr<gfxstream::vk::VkEmulation> sEmulationVk = nullptr; // TODO fix this leakage
 
class GFXStreamThread {
public:
  std::unique_ptr<WebrogueOutputStream> webrogue_output_stream;
  std::unique_ptr<gfxstream::ProcessResources> processResources;
  std::unique_ptr<emugl::GfxApiLogger> gfxLogger;
  std::unique_ptr<android::base::MetricsLogger> metricsLogger;
  gfxstream::vk::VkDecoder vkDec;
  

  GFXStreamThread() {
    // set_emugl_address_space_device_control_ops
    if(!gfxstream::vk::RenderThreadInfoVk::get()) {
      static int ctx_id = 1; // TODO sync
      auto tinfo = new gfxstream::vk::RenderThreadInfoVk(); // TODO fix this leakage
      tinfo->ctx_id = ctx_id++;
    }
    webrogue_output_stream = std::make_unique<WebrogueOutputStream>(16);
    processResources = std::unique_ptr(gfxstream::ProcessResources::create());
    gfxLogger = std::make_unique<emugl::GfxApiLogger>();
    metricsLogger = std::make_unique<MetricsLoggerImpl>();
  }
};

extern "C" {
void webrogue_gfxstream_ffi_create_global_state(void *get_proc, void* userdata) {
  sVkGetProc = reinterpret_cast<get_proc_func_t>(get_proc);
  sVkGetProcUserdata = userdata;
  
  gfxstream::vk::VulkanDispatch* m_vkDispatch = gfxstream::vk::vkDispatch(false);
  gfxstream::host::BackendCallbacks callbacks{
        .registerProcessCleanupCallback =
            [](void* key, std::function<void()> callback) {
                // abort();
                // TODO invoke this callbacks when GFXSystem drops
            },
        .unregisterProcessCleanupCallback =
            [](void* key) { 
              // abort();
            },
        .invalidateColorBuffer =
            [](uint32_t colorBufferHandle) {
              abort();
            },
        .flushColorBuffer =
            [](uint32_t colorBufferHandle) {
              abort();
            },
        .flushColorBufferFromBytes =
            [](uint32_t colorBufferHandle, const void* bytes, size_t bytesSize) {
              abort();
            },
        // .scheduleAsyncWork =
        //     [](std::function<void()> work, std::string description) {
        //       abort();
        //         // auto promise = std::make_shared<AutoCancelingPromise>();
        //         // auto future = promise->GetFuture();
        //         // SyncThread::get()->triggerGeneral(
        //         //     [promise = std::move(promise), work = std::move(work)]() mutable {
        //         //         work();
        //         //         promise->MarkComplete();
        //         //     },
        //         //     description);
        //         // return future;
        //     },
#ifdef CONFIG_AEMU
        .registerVulkanInstance =
            [](uint64_t id, const char* appName) {
              abort();
            },
        .unregisterVulkanInstance =
            [](uint64_t id) { 
              abort();
            },
#endif
    };
  gfxstream::host::FeatureSet features = gfxstream::host::FeatureSet();

  features.VulkanNullOptionalStrings.enabled = true;
  features.VulkanIgnoredHandles.enabled = true;
  features.VulkanShaderFloat16Int8.enabled = true;
  features.VulkanQueueSubmitWithCommands.enabled = true;
  // features.DeferredVulkanCommands.enabled = true;
  // features.VulkanAsyncQueueSubmit.enabled = true;
  // features.VulkanCreateResourcesWithRequirements.enabled = true;
  features.VirtioGpuNext.enabled = true;
  features.VirtioGpuNativeSync.enabled = true;
  features.VulkanBatchedDescriptorSetUpdate.enabled = false; // TODO ?
  // features.VulkanAsyncQsri.enabled = true;

  // ResourceTracker::streamFeatureBits |= VULKAN_STREAM_FEATURE_NULL_OPTIONAL_STRINGS_BIT;
  // ResourceTracker::streamFeatureBits |= VULKAN_STREAM_FEATURE_IGNORED_HANDLES_BIT;
  // ResourceTracker::streamFeatureBits |= VULKAN_STREAM_FEATURE_SHADER_FLOAT16_INT8_BIT;
  // ResourceTracker::streamFeatureBits |= VULKAN_STREAM_FEATURE_QUEUE_SUBMIT_WITH_COMMANDS_BIT;
                                                //  gfxstream::host::BackendCallbacks callbacks,
                                                //  gfxstream::host::FeatureSet features
  sEmulationVk = gfxstream::vk::VkEmulation::create(m_vkDispatch, callbacks, features);
  gfxstream::vk::VkDecoderGlobalState::initialize(sEmulationVk.get());
}

void* webrogue_gfxstream_ffi_create_thread() {
  return new GFXStreamThread();
}

void webrogue_gfxstream_ffi_destroy_thread(void *raw_thread_ptr) {
  GFXStreamThread *thread = (GFXStreamThread *)raw_thread_ptr;
  delete thread;
}

void webrogue_gfxstream_ffi_commit_buffer(void *raw_thread_ptr, void const* buf, uint32_t len) {
  GFXStreamThread *thread = (GFXStreamThread *)raw_thread_ptr;
  WebrogueOutputStream *stream = thread->webrogue_output_stream.get();

  gfxstream::vk::VkDecoderContext context = {
    .processName = "Webrogue contextName idk",
    .gfxApiLogger = thread->gfxLogger.get(),
    .healthMonitor = nullptr,
    .metricsLogger = thread->metricsLogger.get(),
  };
  if(stream->getIncompleteCommitSize()) {
    stream->addIncompleteCommit(buf, len);
    size_t decoded = thread->vkDec.decode(
      stream->getIncompleteCommit(),
      stream->getIncompleteCommitSize(),
      thread->webrogue_output_stream.get(),
      thread->processResources.get(),
      context
    );
    stream->consumeIncompleteCommit(decoded);
  } else {
    size_t decoded = thread->vkDec.decode(
      (void*)buf,
      len,
      thread->webrogue_output_stream.get(),
      thread->processResources.get(),
      context
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
void webrogue_gfxstream_ffi_read_device_memory(void *raw_thread_ptr, void* buf, uint64_t len, uint64_t offset, uint64_t deviceMemory) {
  auto state = gfxstream::vk::VkDecoderGlobalState::get();
  state->webrogue_gfxstream_ffi_read_device_memory(buf, len, offset, deviceMemory);
}
void webrogue_gfxstream_ffi_write_device_memory(void *raw_thread_ptr, void* buf, uint64_t len, uint64_t offset, uint64_t deviceMemory) {
  auto state = gfxstream::vk::VkDecoderGlobalState::get();
  state->webrogue_gfxstream_ffi_write_device_memory(buf, len, offset, deviceMemory);
}
void* webrogue_gfxstream_ffi_unbox_vk_instance(uint64_t vk_instance) {
  VkInstance instance = (VkInstance)vk_instance;
  return gfxstream::vk::unbox_VkInstance(instance);
}
uint64_t webrogue_gfxstream_ffi_box_vk_surface(void *vk_surface) {
  VkSurfaceKHR guest_vk_surface = (VkSurfaceKHR)vk_surface;
  guest_vk_surface = gfxstream::vk::new_boxed_non_dispatchable_VkSurfaceKHR(guest_vk_surface);
  gfxstream::vk::DefaultHandleMapping().mapHandles_VkSurfaceKHR(&guest_vk_surface, 1);
  return (uint64_t)guest_vk_surface;
}
}

static void* sVulkanDispatchDlOpen() {
  return sVkGetProcUserdata;
}

static void* sVulkanDispatchDlSym(void* lib, const char* sym) {
  assert(lib == sVkGetProcUserdata);
  return sVkGetProc(sym, lib);
}

namespace gfxstream {
namespace vk {
gfxstream::vk::VulkanDispatch* vkDispatch(bool forTesting) {
  auto result = new gfxstream::vk::VulkanDispatch();
  gfxstream::vk::init_vulkan_dispatch_from_system_loader(sVulkanDispatchDlOpen, sVulkanDispatchDlSym,  result);
  return result;
}

bool vkDispatchValid(const VulkanDispatch* vk) {
  return vk->vkEnumerateInstanceExtensionProperties != nullptr ||
         vk->vkGetInstanceProcAddr != nullptr || vk->vkGetDeviceProcAddr != nullptr;
}
}
}
