// ! This module is vibecoded. TODO refactor
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
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

/// Renderer flags from `Renderer::new`, kept only for `vkr_get_capset` (the
/// only bit that matters there is `VIRGL_RENDERER_USE_GUEST_VRAM`).
static INIT_FLAGS: AtomicU32 = AtomicU32::new(0);

pub(crate) fn init_flags() -> u32 {
    INIT_FLAGS.load(Ordering::Relaxed)
}

pub(crate) fn init(ctx_flags: c_int) -> c_int {
    // Make sure the fence-id queue exists before venus can call back into it.
    completed().queue.lock().unwrap().clear();
    state();

    INIT_FLAGS.store(ctx_flags as u32, Ordering::Relaxed);

    // Direct dispatch: talk to the venus core (vkr) in-process. No proxy, no
    // socket, no render server worker thread, no fence eventfd. Ring-0 fences
    // retire inline on the submitting thread; GPU-timeline fences retire from
    // vkr's own per-queue sync threads (condvar-based, portable).
    let ok = unsafe {
        bindings::webrogue_vkr_init(
            (bindings::VKR_RENDERER_THREAD_SYNC | bindings::VKR_RENDERER_ASYNC_FENCE_CB) as u32,
            Some(write_context_fence),
        )
    };
    if !ok {
        return -1;
    }
    0
}

unsafe extern "C" fn write_context_fence(_ctx_id: u32, _ring_idx: u32, fence_id: u64) {
    let queue = &completed();
    if let Ok(mut done) = queue.queue.lock() {
        done.push(fence_id);
        queue.cv.notify_all();
    }
}

pub(crate) fn cleanup() {
    context_destroy();
    completed().queue.lock().unwrap().clear();
    unsafe { bindings::vkr_renderer_fini() };
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

    let ok = unsafe {
        bindings::vkr_renderer_create_context(
            ctx.ctx_id,
            ctx.capset_id,
            ctx.debug_name.len() as u32,
            ctx.debug_name.as_ptr() as *const c_char,
        )
    };
    ctx.context_initialized = ok;
    if !ok {
        return -1;
    }
    0
}

pub(crate) fn context_destroy() {
    // Drop any fence completions whose TimelineSubmits are destroyed below.
    completed().queue.lock().unwrap().clear();

    let st = state();
    let mut st = st.lock().unwrap();
    let Some(ctx) = st.context.take() else { return };

    if ctx.context_initialized {
        unsafe { bindings::vkr_renderer_destroy_context(ctx.ctx_id) };
    }
    for res in ctx.resource_table.values() {
        unsafe { bindings::vkr_renderer_destroy_resource(ctx.ctx_id, res.res_id) };
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

    let mut fd_type: bindings::virgl_resource_fd_type = unsafe { std::mem::zeroed() };
    let mut res_fd: c_int = -1;
    let mut map_info: u32 = 0;
    let mut vulkan_info: bindings::virgl_resource_vulkan_info = unsafe { std::mem::zeroed() };
    let mut mapped_ptr: *mut c_void = ptr::null_mut();
    let ok = unsafe {
        bindings::vkr_renderer_create_resource(
            ctx.ctx_id,
            res_id,
            blob_id,
            size as u64,
            if is_shmem {
                bindings::VIRGL_RENDERER_BLOB_FLAG_USE_MAPPABLE
            } else {
                0
            },
            &mut fd_type,
            &mut res_fd,
            &mut map_info,
            &mut vulkan_info,
            &mut mapped_ptr,
        )
    };
    if !ok {
        unsafe { bindings::vkr_renderer_destroy_resource(ctx.ctx_id, res_id) };
        return 0;
    }

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

/// Waits until `poll_and_check` returns `Some(result)`. Fence retirement is
/// fully event-driven in direct dispatch: ring-0 fences retire inline on the
/// submitting thread (before the wait even starts), GPU-timeline fences retire
/// from vkr's per-queue sync threads, and both paths push to `completed()` and
/// notify the condvar, so the waiter wakes immediately. The 1ms timeout is
/// only a safety net for the deadline check in `sync_wait_inner`.
fn wait_completed<T>(st: &mut State, mut poll_and_check: impl FnMut(&mut State) -> Option<T>) -> T {
    loop {
        drain_completed(st);
        if let Some(result) = poll_and_check(st) {
            return result;
        }
        // The queue is consumed by drain_completed() at the top of the next
        // iteration, so entries that arrive while we hold the lock are safe
        // to leave behind.
        let queue = completed().queue.lock().unwrap();
        if queue.is_empty() {
            let _ = completed().cv.wait_timeout(queue, Duration::from_millis(1));
        }
    }
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

    wait_completed(&mut st, |_st| {
        let remaining = targets
            .iter()
            .filter(|(sync, value)| sync.value.load(Ordering::Relaxed) < *value)
            .count();
        let ready = remaining == 0 || ((flags & SYNC_WAIT_FLAG_ANY) != 0 && remaining < sync_count);
        if ready {
            return Some(0);
        }
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                return Some(2); // VK_TIMEOUT
            }
        }
        None
    })
}

pub(crate) fn submit_cmd(headers: &[u32], cmds: &[u32], syncs: &[u32]) -> c_int {
    let st = state();
    let mut st = st.lock().unwrap();

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

    // ring_idx 0 is the CPU timeline: vkr retires its fences inline, right
    // after the batch has been dispatched and all replies written, so the
    // collected fence ids below are already retired by the time this call
    // returns and the wait at the bottom finds them drained immediately.
    // GPU timelines (ring_idx > 0) retire asynchronously from vkr's per-queue
    // sync threads.
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

        let ok = unsafe {
            bindings::vkr_renderer_submit_cmd(
                ctx.ctx_id,
                cmds.as_ptr().add(cmd_offset) as *mut c_void,
                (cmd_size * std::mem::size_of::<u32>()) as u32,
            )
        };
        if !ok {
            return -1;
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

        let ok = unsafe {
            bindings::vkr_renderer_submit_fence(
                ctx.ctx_id,
                bindings::VIRGL_RENDERER_FENCE_FLAG_MERGEABLE,
                ring as u64,
                fence_id,
            )
        };
        if !ok {
            ctx.timelines[ring].pop();
            return -1;
        }
        if ring == 0 {
            cpu_fence_ids.push(fence_id);
        }
    }

    if !cpu_fence_ids.is_empty() {
        wait_completed(&mut st, |st| {
            let Some(ctx) = st.context.as_ref() else {
                return Some(0);
            };
            let all_done = !cpu_fence_ids.iter().any(|id| {
                ctx.timelines[0].iter().any(|s| {
                    let p: *const TimelineSubmit = &**s;
                    std::ptr::eq(p, *id as usize as *const TimelineSubmit)
                })
            });
            all_done.then_some(0)
        });
    }

    drain_completed(&mut st);
    0
}
