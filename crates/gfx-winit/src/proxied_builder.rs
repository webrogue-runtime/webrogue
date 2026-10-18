use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak, atomic::AtomicBool},
};

use webrogue_gfx::System;
use winit::{
    event::{FingerId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{WindowAttributes, WindowId},
};

use crate::{
    WinitSystem, WinitWindow,
    mailbox::{Mailbox, RequestQueue},
    surface_lifecycle::{SURFACE_DESTROY_TIMEOUT, SurfaceLifecycle},
    utils::lock_mutex,
    window_registry::WindowRegistry,
};

pub struct ProxiedWinitBuilder {
    proxy: WinitProxy,
    window_attributes_fn: Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
}

impl ProxiedWinitBuilder {
    pub fn new(event_loop_proxy: EventLoopProxy) -> (Self, WinitProxy) {
        let mailbox = Mailbox {
            event_loop_proxy,
            requests: Arc::new(Mutex::new(Default::default())),
            surface_requests: Arc::new(Mutex::new(Default::default())),
            deferred_requests: Arc::new(Mutex::new(Default::default())),
            surfaces_available: Arc::new(AtomicBool::new(false)),
        };
        let vk_windows = Arc::new(Mutex::new(Vec::new()));
        let surface_lifecycle = Arc::new(SurfaceLifecycle::default());
        let proxy = WinitProxy {
            internal: Arc::new(Mutex::new(WinitProxyInternal {
                mailbox,
                on_hide: None,
                vk_windows,
                surface_lifecycle,
                surface_callbacks: SurfaceCallbackState::default(),
                finger_id_map: BTreeMap::new(),
                simulate_touch: Arc::new(AtomicBool::new(true)),
            })),
        };
        (
            Self {
                proxy: proxy.clone(),
                window_attributes_fn: None,
            },
            proxy,
        )
    }

    /// Set the fallback called if the guest does not destroy all Vulkan
    /// surfaces before the lifecycle timeout. The callback must terminate the
    /// process; returning still aborts to avoid returning to Android with a
    /// live surface.
    pub fn with_on_hide(self, on_hide: Box<dyn Fn() + Send + Sync + 'static>) -> Self {
        self.proxy.internal.lock().unwrap().on_hide = Some(Arc::from(on_hide));
        self
    }

    pub fn with_window_attributes(
        mut self,
        window_attributes_fn: impl Fn(WindowAttributes) -> WindowAttributes + Send + Sync + 'static,
    ) -> Self {
        self.window_attributes_fn = Some(Arc::new(window_attributes_fn));
        self
    }

    pub fn to_system(self, vulkan_requirement: Option<bool>) -> anyhow::Result<System> {
        let (mailbox, vk_windows, surface_lifecycle) = {
            let proxy = self.proxy.internal.lock().unwrap();
            (
                proxy.mailbox.clone(),
                proxy.vk_windows.clone(),
                proxy.surface_lifecycle.clone(),
            )
        };
        let system = WinitSystem::new(
            mailbox,
            vulkan_requirement,
            self.window_attributes_fn.clone(),
            vk_windows,
            surface_lifecycle,
        )?;
        let system = System::new(Arc::new(system));
        Ok(system)
    }
}
struct WinitProxyInternal {
    mailbox: Mailbox,
    on_hide: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    vk_windows: Arc<Mutex<Vec<Weak<WinitWindow>>>>,
    surface_lifecycle: Arc<SurfaceLifecycle>,
    surface_callbacks: SurfaceCallbackState,
    finger_id_map: BTreeMap<FingerId, u32>,
    simulate_touch: Arc<AtomicBool>,
}

#[derive(Default)]
struct SurfaceCallbackState {
    can_create_surfaces_received: bool,
    surfaces_were_destroyed: bool,
}

impl SurfaceCallbackState {
    fn on_can_create_surfaces(&mut self) -> bool {
        self.can_create_surfaces_received = true;
        std::mem::replace(&mut self.surfaces_were_destroyed, false)
    }

    fn on_destroy_surfaces(&mut self) -> bool {
        if self.surfaces_were_destroyed {
            return false;
        }
        self.surfaces_were_destroyed = true;
        true
    }

    fn can_simulate_destroy_surfaces(&self) -> bool {
        self.can_create_surfaces_received && !self.surfaces_were_destroyed
    }

    fn can_simulate_can_create_surfaces(&self) -> bool {
        self.can_create_surfaces_received && self.surfaces_were_destroyed
    }
}

#[derive(Clone)]
pub struct WinitProxy {
    internal: Arc<Mutex<WinitProxyInternal>>,
}

impl WinitProxy {
    pub fn downgrade(&self) -> WeakWinitProxy {
        WeakWinitProxy {
            internal: Arc::downgrade(&self.internal),
        }
    }

    pub fn proxy_wake_up(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        let mailbox = lock_mutex(&self.internal).mailbox.clone();
        Self::run_requests(&mailbox.requests, event_loop, window_registry);
        if mailbox.surfaces_available() {
            Self::run_requests(&mailbox.deferred_requests, event_loop, window_registry);
            Self::run_requests(&mailbox.surface_requests, event_loop, window_registry);
        }
    }

    pub fn can_create_surfaces(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        let (mailbox, vk_windows, surfaces_were_destroyed) = {
            let mut internal = lock_mutex(&self.internal);
            let surfaces_were_destroyed = internal.surface_callbacks.on_can_create_surfaces();
            (
                internal.mailbox.clone(),
                internal.vk_windows.clone(),
                surfaces_were_destroyed,
            )
        };
        let attached_windows = if surfaces_were_destroyed {
            let windows = vk_windows
                .lock()
                .unwrap()
                .iter()
                .filter_map(Weak::upgrade)
                .filter(|window| window.had_native_window())
                .collect::<Vec<_>>();
            for window in &windows {
                window.create_native_window(event_loop, window_registry);
            }
            windows
        } else {
            Vec::new()
        };

        mailbox.resume_surfaces();
        self.proxy_wake_up(event_loop, window_registry);
        for window in attached_windows {
            window
                .event_sink
                .push_critical_event(webrogue_gfx::events::Event::SurfaceAttached);
            window
                .event_sink
                .push_event(webrogue_gfx::events::Event::RedrawRequested);
        }
    }

    pub fn destroy_surfaces(
        &self,
        _event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        let (on_hide, surface_lifecycle, vk_windows) = {
            let mut internal = lock_mutex(&self.internal);
            if !internal.surface_callbacks.on_destroy_surfaces() {
                return;
            }
            internal.mailbox.suspend_surfaces();
            (
                internal.on_hide.clone(),
                internal.surface_lifecycle.clone(),
                internal.vk_windows.clone(),
            )
        };

        // Complete operations queued just before suspension with their
        // unavailable/no-op result; do not leave the guest blocked while it
        // needs to process SurfaceLost.
        let mailbox = lock_mutex(&self.internal).mailbox.clone();
        Self::run_requests(&mailbox.surface_requests, _event_loop, window_registry);

        let affected_window_ids = window_registry.notify_surface_lost();
        if !surface_lifecycle.wait_until_destroyed(&affected_window_ids, SURFACE_DESTROY_TIMEOUT) {
            if let Some(on_hide) = on_hide {
                on_hide();
            }
            // Returning to Android with a live VkSurfaceKHR would let the OS
            // invalidate its native window while Vulkan still owns it.
            std::process::abort();
        }
        for window in vk_windows.lock().unwrap().iter().filter_map(Weak::upgrade) {
            window.detach_native_window();
        }
        window_registry.clean();
    }

    pub fn simulate_destroy_surfaces(&self) {
        let weak_proxy = self.downgrade();
        self.get_mailbox()
            .execute(move |event_loop, window_registry| {
                let Some(proxy) = weak_proxy.upgrade() else {
                    return;
                };
                let should_simulate = lock_mutex(&proxy.internal)
                    .surface_callbacks
                    .can_simulate_destroy_surfaces();
                if should_simulate {
                    proxy.destroy_surfaces(event_loop, window_registry);
                }
            });
    }

    pub fn simulate_can_create_surfaces(&self) {
        let weak_proxy = self.downgrade();
        self.get_mailbox()
            .execute(move |event_loop, window_registry| {
                let Some(proxy) = weak_proxy.upgrade() else {
                    return;
                };
                let should_simulate = lock_mutex(&proxy.internal)
                    .surface_callbacks
                    .can_simulate_can_create_surfaces();
                if should_simulate {
                    proxy.can_create_surfaces(event_loop, window_registry);
                }
            });
    }

    fn run_requests(
        requests: &RequestQueue,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        loop {
            let func = requests.lock().unwrap().pop_front();
            let Some(func) = func else {
                return;
            };
            func(event_loop, window_registry);
        }
    }

    pub fn window_event(
        &self,
        window_registry: &mut WindowRegistry,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let mut internal = self.internal.lock().unwrap();
        let simulate_touch = internal.simulate_touch.clone();
        if let Some(window) = window_registry.get_window_by_winit_id(window_id) {
            window.on_event(
                event,
                &mut internal.finger_id_map,
                simulate_touch.load(std::sync::atomic::Ordering::Acquire),
            );
        }
    }

    pub(crate) fn get_mailbox(&self) -> Mailbox {
        self.internal.lock().unwrap().mailbox.clone()
    }
}

#[derive(Clone)]
pub struct WeakWinitProxy {
    internal: Weak<Mutex<WinitProxyInternal>>,
}

impl WeakWinitProxy {
    pub fn upgrade(&self) -> Option<WinitProxy> {
        self.internal
            .upgrade()
            .map(|internal| WinitProxy { internal })
    }
}

impl webrogue_gfx::AbstractBuilder for ProxiedWinitBuilder {
    fn run<Output>(
        self,
        body_fn: impl FnOnce(System) -> Output + Send + 'static,
        vulkan_requirement: Option<bool>,
    ) -> anyhow::Result<Output>
    where
        Output: Send + 'static,
    {
        let system = self.to_system(vulkan_requirement)?;

        Ok(body_fn(system))
    }
}

#[cfg(test)]
mod tests {
    use super::SurfaceCallbackState;

    #[test]
    fn simulated_attach_waits_for_the_first_real_availability_callback() {
        let mut state = SurfaceCallbackState::default();

        assert!(!state.can_simulate_can_create_surfaces());
        assert!(!state.can_simulate_destroy_surfaces());

        assert!(!state.on_can_create_surfaces());
        assert!(state.can_simulate_destroy_surfaces());
    }

    #[test]
    fn duplicate_destroy_is_ignored_and_attach_resets_the_state() {
        let mut state = SurfaceCallbackState::default();
        state.on_can_create_surfaces();

        assert!(state.on_destroy_surfaces());
        assert!(!state.on_destroy_surfaces());
        assert!(state.can_simulate_can_create_surfaces());

        assert!(state.on_can_create_surfaces());
        assert!(!state.can_simulate_can_create_surfaces());
        assert!(state.on_destroy_surfaces());
    }

    #[test]
    fn simulated_attach_does_not_run_for_a_destroy_before_initial_availability() {
        let mut state = SurfaceCallbackState::default();

        assert!(state.on_destroy_surfaces());
        assert!(!state.can_simulate_can_create_surfaces());
        assert!(state.on_can_create_surfaces());
        assert!(!state.can_simulate_can_create_surfaces());
    }
}
