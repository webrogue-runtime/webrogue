use std::collections::HashMap;
use std::ffi::{c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::bindings;

#[cfg(unix)]
unsafe fn unmap(ptr: *mut c_void, len: usize) {
    libc::munmap(ptr, len);
}

#[cfg(windows)]
unsafe fn unmap(ptr: *mut c_void, _len: usize) {
    use windows_sys::Win32::System::Memory::{VirtualFree, MEM_RELEASE};
    VirtualFree(ptr as *mut _, 0, MEM_RELEASE);
}

pub(crate) const MAX_TIMELINE_COUNT: usize = 64;
const SYNC_WAIT_FLAG_ANY: u32 = 1;

const EINVAL: i32 = 22;
const EEXIST: i32 = 17;

struct Sync {
    value: AtomicU64,
}

struct TimelineSubmit {
    ring_idx: u32,
    syncs: Vec<Arc<Sync>>,
    values: Vec<u64>,
}

struct Resource {
    res_id: u32,
    iov: Option<(usize, usize)>,
}

struct Context {
    ctx_id: u32,
    debug_name: Vec<u8>,
    capset_id: u32,
    context_initialized: bool,
    next_resource_id: u32,
    next_sync_id: u32,
    resource_table: HashMap<u32, Resource>,
    sync_table: HashMap<u32, Arc<Sync>>,
    timelines: Vec<Vec<Box<TimelineSubmit>>>,
}

struct State {
    context: Option<Context>,
}

fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State { context: None }))
}

fn completed() -> &'static CompletedQueue {
    static COMPLETED: OnceLock<CompletedQueue> = OnceLock::new();
    COMPLETED.get_or_init(|| CompletedQueue {
        queue: Mutex::new(Vec::new()),
        cv: Condvar::new(),
    })
}

/// Fence-completion queue paired with a condvar so host-side waiters
/// (`sync_wait`) wake up the moment `write_context_fence` fires instead of
/// polling at the OS sleep granularity. Windows `Sleep(1)` is ~15.6 ms, so the
/// old 1 ms poll loop was actually sampling at ~64 Hz and every fence took at
/// least one full sleep quantum per wait.
struct CompletedQueue {
    queue: Mutex<Vec<u64>>,
    cv: Condvar,
}

fn decode_triplet(data: &[u32], index: usize) -> (u32, u64) {
    let base = index * 3;
    let id = data[base];
    let lo = data[base + 1] as u64;
    let hi = data[base + 2] as u64;
    (id, lo | (hi << 32))
}

fn signal_sync(sync: &Arc<Sync>, value: u64) {
    if sync.value.load(Ordering::Relaxed) >= value {
        return;
    }
    sync.value.store(value, Ordering::Relaxed);
}

/// Processes fences completed by venus. Each completed fence id points to a
/// `TimelineSubmit` still owned by its timeline; we signal every submit on that
/// timeline up to and including the completed one (MERGEABLE fences imply earlier
/// submits).
fn drain_completed(st: &mut State) {
    let ids = std::mem::take(&mut *completed().queue.lock().unwrap());
    if !ids.is_empty() {
        completed().cv.notify_all();
    }
    if ids.is_empty() {
        return;
    }
    let Some(ctx) = st.context.as_mut() else {
        return;
    };
    for fence_id in ids {
        let ptr = fence_id as usize as *const TimelineSubmit;
        if ptr.is_null() {
            continue;
        }
        let ring = unsafe { (*ptr).ring_idx } as usize;
        if ring >= MAX_TIMELINE_COUNT {
            continue;
        }
        let timeline = &mut ctx.timelines[ring];
        let Some(pos) = timeline.iter().position(|s| {
            let p: *const TimelineSubmit = &**s;
            std::ptr::eq(p, ptr)
        }) else {
            continue;
        };
        let done: Vec<Box<TimelineSubmit>> = timeline.drain(..=pos).collect();
        for submit in done {
            for (sync, value) in submit.syncs.iter().zip(submit.values.iter()) {
                signal_sync(sync, *value);
            }
        }
    }
}

pub(crate) fn init(ctx_flags: c_int) -> c_int {
    // Make sure the fence-id queue exists before venus can call back into it.
    completed().queue.lock().unwrap().clear();
    state();

    static CALLBACKS: OnceLock<Box<bindings::virgl_renderer_callbacks>> = OnceLock::new();
    let callbacks = CALLBACKS.get_or_init(|| {
        unsafe extern "C" fn write_fence(_cookie: *mut c_void, _fence: u32) {}
        unsafe extern "C" fn get_drm_fd(_cookie: *mut c_void) -> c_int {
            -1
        }
        unsafe extern "C" fn write_context_fence(
            _cookie: *mut c_void,
            _ctx_id: u32,
            _ring_idx: u32,
            fence_id: u64,
        ) {
            let queue = &completed();
            if let Ok(mut done) = queue.queue.lock() {
                done.push(fence_id);
                queue.cv.notify_all();
            }
        }
        Box::new(bindings::virgl_renderer_callbacks {
            version: bindings::VIRGL_RENDERER_CALLBACKS_VERSION as c_int,
            write_fence: Some(write_fence),
            create_gl_context: None,
            destroy_gl_context: None,
            make_current: None,
            get_drm_fd: Some(get_drm_fd),
            write_context_fence: Some(write_context_fence),
            get_server_fd: None,
            get_egl_display: None,
        })
    });

    let flags = ctx_flags
        | bindings::VIRGL_RENDERER_THREAD_SYNC as c_int
        | bindings::VIRGL_RENDERER_USE_EXTERNAL_BLOB as c_int;

    let cb: *mut bindings::virgl_renderer_callbacks = &**callbacks as *const _ as *mut _;
    let ret = unsafe { bindings::virgl_renderer_init(ptr::null_mut(), flags, cb) };
    if ret != 0 {
        return -1;
    }
    0
}

pub(crate) fn cleanup() {
    context_destroy();
    completed().queue.lock().unwrap().clear();
    unsafe { bindings::virgl_renderer_cleanup(ptr::null_mut()) };
}

pub(crate) fn context_create(name: &[u8]) -> c_int {
    let st = state();
    let mut st = st.lock().unwrap();
    if st.context.is_some() {
        return -1;
    }
    if name.len() > 1024 * 1024 {
        return -1;
    }

    let mut debug_name = name.to_vec();
    if debug_name.last() != Some(&0) {
        debug_name.push(0);
    }
    st.context = Some(Context {
        ctx_id: 1,
        debug_name,
        capset_id: 0,
        context_initialized: false,
        next_resource_id: 1,
        next_sync_id: 1,
        resource_table: HashMap::new(),
        sync_table: HashMap::new(),
        timelines: (0..MAX_TIMELINE_COUNT).map(|_| Vec::new()).collect(),
    });
    0
}

pub(crate) fn context_init(capset_id: u32) -> c_int {
    let st = state();
    let mut st = st.lock().unwrap();
    let Some(ctx) = st.context.as_mut() else {
        return -1;
    };
    if capset_id == 0 {
        return -EINVAL;
    }
    if ctx.context_initialized {
        return if ctx.capset_id == capset_id {
            0
        } else {
            -EINVAL
        };
    }
    ctx.capset_id = capset_id;

    let ret = unsafe {
        bindings::virgl_renderer_context_create_with_flags(
            ctx.ctx_id,
            ctx.capset_id,
            ctx.debug_name.len() as u32,
            ctx.debug_name.as_ptr() as *const i8,
        )
    };
    ctx.context_initialized = ret == 0;
    ret
}

pub(crate) fn context_destroy() {
    // Drop any fence completions whose TimelineSubmits are destroyed below.
    completed().queue.lock().unwrap().clear();

    let st = state();
    let mut st = st.lock().unwrap();
    let Some(ctx) = st.context.take() else { return };

    if ctx.context_initialized {
        unsafe { bindings::virgl_renderer_context_destroy(ctx.ctx_id) };
    }
    for res in ctx.resource_table.values() {
        unsafe { bindings::virgl_renderer_resource_unref(res.res_id) };
        if let Some((ptr, len)) = res.iov {
            unsafe { unmap(ptr as *mut c_void, len) };
        }
    }
    // timelines (TimelineSubmits) and hash tables are dropped here.
}

pub(crate) fn create_blob(ptr: usize, size: usize, blob_id: u64) -> u32 {
    let st = state();
    let mut st = st.lock().unwrap();
    let Some(ctx) = st.context.as_mut() else {
        return 0;
    };

    let res_id = ctx.next_resource_id;
    ctx.next_resource_id += 1;

    let is_shmem = blob_id == 0;
    let iov = if is_shmem {
        Some(bindings::iovec {
            iov_base: ptr as *mut c_void,
            iov_len: size,
        })
    } else {
        None
    };
    let args = bindings::virgl_renderer_resource_create_blob_args {
        res_handle: res_id,
        ctx_id: ctx.ctx_id,
        blob_mem: if is_shmem {
            bindings::VIRGL_RENDERER_BLOB_MEM_HOST3D_GUEST
        } else {
            bindings::VIRGL_RENDERER_BLOB_MEM_HOST3D
        },
        blob_flags: if is_shmem {
            bindings::VIRGL_RENDERER_BLOB_FLAG_USE_MAPPABLE
        } else {
            0
        },
        blob_id,
        size: size as u64,
        iovecs: iov.as_ref().map_or(ptr::null(), |i| i),
        num_iovs: if is_shmem { 1 } else { 0 },
    };

    let ret = unsafe { bindings::virgl_renderer_resource_create_blob(&args) };
    if ret != 0 {
        if let Some(i) = iov {
            unsafe { unmap(i.iov_base, i.iov_len) };
        }
        unsafe { bindings::virgl_renderer_resource_unref(res_id) };
        return 0;
    }

    unsafe { bindings::virgl_renderer_ctx_attach_resource(ctx.ctx_id as c_int, res_id as c_int) };
    ctx.resource_table.insert(
        res_id,
        Resource {
            res_id,
            iov: if is_shmem { Some((ptr, size)) } else { None },
        },
    );

    res_id
}

pub(crate) fn resource_unref(res_id: u32) {
    let st = state();
    let mut st = st.lock().unwrap();
    if let Some(ctx) = st.context.as_mut() {
        ctx.resource_table.remove(&res_id);
    }
}

pub(crate) fn sync_create(value: u64) -> u32 {
    let st = state();
    let mut st = st.lock().unwrap();
    let Some(ctx) = st.context.as_mut() else {
        return 0;
    };
    let id = ctx.next_sync_id;
    ctx.next_sync_id += 1;
    ctx.sync_table.insert(
        id,
        Arc::new(Sync {
            value: AtomicU64::new(value),
        }),
    );
    id
}

pub(crate) fn sync_unref(sync_id: u32) {
    let st = state();
    let mut st = st.lock().unwrap();
    if let Some(ctx) = st.context.as_mut() {
        ctx.sync_table.remove(&sync_id);
    }
}

pub(crate) fn sync_read(sync_id: u32) -> u64 {
    let st = state();
    let mut st = st.lock().unwrap();
    let Some(ctx) = st.context.as_mut() else {
        return 0;
    };
    ctx.sync_table
        .get(&sync_id)
        .map_or(0, |s| s.value.load(Ordering::Relaxed))
}

pub(crate) fn sync_write(sync_id: u32, value: u64) -> c_int {
    let st = state();
    let mut st = st.lock().unwrap();
    let Some(ctx) = st.context.as_mut() else {
        return -1;
    };
    let Some(sync) = ctx.sync_table.get(&sync_id).cloned() else {
        return -EEXIST;
    };
    signal_sync(&sync, value);
    0
}

/// Waits (blocking, on the host) until the given syncs reach their target
/// values, the timeout elapses, or, with `SYNC_WAIT_FLAG_ANY`, any single one
/// does. Host-side blocking is required because host fds can't be shared with
/// the guest. The render server makes in-flight work complete asynchronously,
/// so this loop just polls it and re-checks.
///
/// Returns 0 (ready), 2 (VK_TIMEOUT), or a negative errno on error.
pub(crate) fn sync_wait(flags: u32, timeout_ms: u32, syncs: &[u32]) -> i32 {
    sync_wait_inner(flags, timeout_ms, syncs)
}

fn sync_wait_inner(flags: u32, timeout_ms: u32, syncs: &[u32]) -> i32 {
    let st = state();
    let mut st = st.lock().unwrap();

    let Some(ctx) = st.context.as_mut() else {
        return -1;
    };

    if syncs.len() % 3 != 0 {
        return -EINVAL;
    }
    let sync_count = syncs.len() / 3;

    let mut targets: Vec<(Arc<Sync>, u64)> = Vec::with_capacity(sync_count);
    for i in 0..sync_count {
        let (sync_id, value) = decode_triplet(syncs, i);
        match ctx.sync_table.get(&sync_id) {
            Some(s) => targets.push((s.clone(), value)),
            None => return -EEXIST,
        }
    }

    let ready = |remaining: usize| -> bool {
        remaining == 0 || ((flags & SYNC_WAIT_FLAG_ANY) != 0 && remaining < sync_count)
    };
    // Poll virgl so in-flight work completes and fires fence callbacks, then
    // drain the completed fences into sync values.
    let poll_and_check = |st: &mut State| -> bool {
        unsafe { bindings::virgl_renderer_poll() };
        drain_completed(st);
        let remaining = targets
            .iter()
            .filter(|(sync, value)| sync.value.load(Ordering::Relaxed) < *value)
            .count();
        ready(remaining)
    };

    if poll_and_check(&mut st) {
        return 0;
    }

    // u32::MAX from the guest means wait forever.
    let deadline = if timeout_ms == u32::MAX {
        None
    } else {
        Some(
            Instant::now()
                .checked_add(Duration::from_millis(timeout_ms as u64))
                .unwrap(),
        )
    };

    loop {
        if poll_and_check(&mut st) {
            return 0;
        }
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                return 2; // VK_TIMEOUT
            }
        }
        {
            // Wait for a fence-completion notification instead of blind-sleeping.
            // On Windows `Sleep(1)` quantizes to ~15.6 ms, so a fence that
            // completes between polls previously cost a full sleep quantum per
            // check; the condvar wakes us the moment the fence callback fires.
            // The queue is consumed by drain_completed() at the top of the next
            // iteration, so entries that arrive while we hold the lock are safe
            // to leave behind.
            let queue = completed().queue.lock().unwrap();
            if queue.is_empty() {
                let _ = completed().cv.wait_timeout(queue, Duration::from_millis(1));
            }
        }
    }
}

pub(crate) fn submit_cmd(headers: &[u32], cmds: &[u32], syncs: &[u32]) -> c_int {
    let st = state();
    let mut st = st.lock().unwrap();

    unsafe { bindings::virgl_renderer_poll() };
    drain_completed(&mut st);

    let Some(ctx) = st.context.as_mut() else {
        return -1;
    };

    // Each batch header is 5 dwords: cmd_offset, cmd_size, sync_offset,
    // sync_count, ring_idx.
    if headers.is_empty() || headers.len() % 5 != 0 {
        return -EINVAL;
    }
    let batch_count = headers.len() / 5;

    let mut cpu_fence_ids: Vec<u64> = Vec::new();

    for bi in 0..batch_count {
        let h = &headers[bi * 5..bi * 5 + 5];
        let cmd_offset = h[0] as usize;
        let cmd_size = h[1] as usize;
        let sync_offset = h[2] as usize;
        let sync_count = h[3] as usize;
        let ring = h[4] as usize;

        if cmd_offset + cmd_size > cmds.len()
            || sync_offset + sync_count * 3 > syncs.len()
            || ring >= MAX_TIMELINE_COUNT
        {
            return -EINVAL;
        }

        let ret = unsafe {
            bindings::virgl_renderer_submit_cmd(
                cmds.as_ptr().add(cmd_offset) as *mut c_void,
                ctx.ctx_id as c_int,
                cmd_size as c_int,
            )
        };
        if ret != 0 {
            return ret;
        }

        if ring != 0 && sync_count == 0 {
            continue;
        }

        let mut sub = TimelineSubmit {
            ring_idx: ring as u32,
            syncs: Vec::new(),
            values: Vec::new(),
        };
        for si in 0..sync_count {
            let (sync_id, value) = decode_triplet(syncs, sync_offset + si * 3);
            match ctx.sync_table.get(&sync_id) {
                Some(s) => {
                    sub.syncs.push(s.clone());
                    sub.values.push(value);
                }
                None => return -EEXIST,
            }
        }

        let boxed = Box::new(sub);
        let fence_id = (&*boxed as *const TimelineSubmit) as usize as u64;
        ctx.timelines[ring].push(boxed);

        let ret = unsafe {
            bindings::virgl_renderer_context_create_fence(
                ctx.ctx_id,
                bindings::VIRGL_RENDERER_FENCE_FLAG_MERGEABLE,
                ring as u32,
                fence_id,
            )
        };
        if ret != 0 {
            ctx.timelines[ring].pop();
            return ret;
        }
        if ring == 0 {
            cpu_fence_ids.push(fence_id);
        }
    }

    if !cpu_fence_ids.is_empty() {
        loop {
            unsafe { bindings::virgl_renderer_poll() };
            drain_completed(&mut st);

            let all_done = {
                let Some(ctx) = st.context.as_ref() else {
                    return -1;
                };
                !cpu_fence_ids.iter().any(|id| {
                    ctx.timelines[0].iter().any(|s| {
                        let p: *const TimelineSubmit = &**s;
                        std::ptr::eq(p, *id as usize as *const TimelineSubmit)
                    })
                })
            };
            if all_done {
                break;
            }

            {
                let queue = completed().queue.lock().unwrap();
                if queue.is_empty() {
                    let _ = completed().cv.wait_timeout(queue, Duration::from_millis(1));
                }
            }
        }
    }

    drain_completed(&mut st);
    0
}
