use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak, atomic::AtomicBool},
};

use webrogue_gfx::{System, events::Event};
use winit::{
    event::{FingerId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{WindowAttributes, WindowId},
};

use crate::{
    WinitSystem, WinitWindow,
    lifecycle_state::{LifecycleState, SurfaceAvailability, SurfaceTeardown},
    mailbox::{Mailbox, RequestQueue},
    surface_lifecycle::{SURFACE_DESTROY_TIMEOUT, SurfaceLifecycle},
    utils::lock_mutex,
    window_icons::{WindowIcons, set_window_icon},
    window_registry::WindowRegistry,
};

pub struct ProxiedWinitBuilder {
    proxy: WinitProxy,
    window_attributes_fn: Option<Arc<dyn Fn(WindowAttributes) -> WindowAttributes + Send + Sync>>,
}

impl ProxiedWinitBuilder {
    pub fn new(event_loop_proxy: EventLoopProxy) -> (Self, WinitProxy) {
        let proxy = WinitProxy {
            state: Arc::new(Mutex::new(WinitProxyState::new(event_loop_proxy))),
        };
        (
            Self {
                proxy: proxy.clone(),
                window_attributes_fn: None,
            },
            proxy,
        )
    }

    /// Set the callback invoked when surface teardown times out, i.e. when the
    /// guest does not destroy all Vulkan surfaces before the lifecycle timeout.
    /// The callback must terminate the process; returning still aborts to avoid
    /// returning to Android with a live surface.
    pub fn with_on_surface_teardown_timeout(
        self,
        on_surface_teardown_timeout: Box<dyn Fn() + Send + Sync + 'static>,
    ) -> Self {
        self.proxy.with_state(|state| {
            state.on_surface_teardown_timeout = Some(Arc::from(on_surface_teardown_timeout));
        });
        self
    }

    pub fn with_window_attributes(
        mut self,
        window_attributes_fn: impl Fn(WindowAttributes) -> WindowAttributes + Send + Sync + 'static,
    ) -> Self {
        self.window_attributes_fn = Some(Arc::new(window_attributes_fn));
        self
    }

    pub fn with_simulate_touch(self, simulate_touch: bool) -> Self {
        self.proxy
            .with_state(|state| state.simulate_touch = simulate_touch);
        self
    }

    pub fn with_window_icons(self, icons: WindowIcons) -> Self {
        self.proxy
            .with_state(|state| *state.icons.lock().unwrap() = icons);
        self
    }

    pub fn to_system(self, vulkan_requirement: Option<bool>) -> anyhow::Result<System> {
        let (mailbox, vk_windows, surface_lifecycle, icons) = self.proxy.with_state(|state| {
            (
                state.mailbox.clone(),
                state.vk_windows.clone(),
                state.surface_lifecycle.clone(),
                state.icons.clone(),
            )
        });
        let system = WinitSystem::new(
            mailbox,
            vulkan_requirement,
            self.window_attributes_fn.clone(),
            vk_windows,
            surface_lifecycle,
            icons,
        )?;
        let system = System::new(Arc::new(system));
        Ok(system)
    }

    pub fn run<Output>(
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

struct WinitProxyState {
    mailbox: Mailbox,
    on_surface_teardown_timeout: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    vk_windows: Arc<Mutex<Vec<Weak<WinitWindow>>>>,
    surface_lifecycle: Arc<SurfaceLifecycle>,
    lifecycle_state: LifecycleState,
    finger_id_map: BTreeMap<FingerId, u32>,
    simulate_touch: bool,
    icons: Arc<Mutex<WindowIcons>>,
}

impl WinitProxyState {
    fn new(event_loop_proxy: EventLoopProxy) -> Self {
        Self {
            mailbox: Mailbox {
                event_loop_proxy,
                requests: Arc::new(Mutex::new(Default::default())),
                surface_requests: Arc::new(Mutex::new(Default::default())),
                deferred_requests: Arc::new(Mutex::new(Default::default())),
                surfaces_available: Arc::new(AtomicBool::new(false)),
            },
            on_surface_teardown_timeout: None,
            vk_windows: Arc::new(Mutex::new(Vec::new())),
            surface_lifecycle: Arc::new(SurfaceLifecycle::default()),
            lifecycle_state: LifecycleState::default(),
            finger_id_map: BTreeMap::new(),
            simulate_touch: false,
            icons: Arc::new(Mutex::new(WindowIcons {
                light: None,
                dark: None,
            })),
        }
    }
}

#[derive(Clone, Copy)]
enum LifecycleAction {
    Availability(SurfaceAvailability),
    Teardown(SurfaceTeardown),
}

#[derive(Clone)]
pub struct WinitProxy {
    state: Arc<Mutex<WinitProxyState>>,
}

impl WinitProxy {
    fn with_state<R>(&self, f: impl FnOnce(&mut WinitProxyState) -> R) -> R {
        let mut state = lock_mutex(&self.state);
        f(&mut state)
    }

    pub fn downgrade(&self) -> WeakWinitProxy {
        WeakWinitProxy {
            state: Arc::downgrade(&self.state),
        }
    }

    pub fn proxy_wake_up(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        let mailbox = self.get_mailbox();
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
        self.handle_platform_lifecycle_event(event_loop, window_registry, |state| {
            LifecycleAction::Availability(state.platform_available())
        });
    }

    pub fn destroy_surfaces(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        self.handle_platform_lifecycle_event(event_loop, window_registry, |state| {
            LifecycleAction::Teardown(state.platform_lost())
        });
    }

    fn handle_platform_lifecycle_event(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
        update: impl FnOnce(&mut LifecycleState) -> LifecycleAction,
    ) {
        let action = self.update_lifecycle_state(update);
        self.apply_lifecycle_action(event_loop, window_registry, action);
    }

    fn update_lifecycle_state(
        &self,
        update: impl FnOnce(&mut LifecycleState) -> LifecycleAction,
    ) -> LifecycleAction {
        self.with_state(|state| update(&mut state.lifecycle_state))
    }

    fn apply_lifecycle_action(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
        action: LifecycleAction,
    ) {
        match action {
            LifecycleAction::Availability(availability) => {
                self.apply_availability(event_loop, window_registry, availability);
            }
            LifecycleAction::Teardown(teardown) => {
                self.apply_teardown(event_loop, window_registry, teardown);
            }
        }
    }

    fn apply_availability(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
        availability: SurfaceAvailability,
    ) {
        match availability {
            SurfaceAvailability::Ignored => {}
            SurfaceAvailability::ResumeOnly => {
                self.resume_surfaces(event_loop, window_registry);
            }
            SurfaceAvailability::RecreateAndResume => {
                let windows = self.recreate_native_windows(event_loop, window_registry);
                self.resume_surfaces(event_loop, window_registry);
                Self::notify_surfaces_attached(windows);
            }
        }
    }

    fn recreate_native_windows(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) -> Vec<Arc<WinitWindow>> {
        let (vk_windows, icons) =
            self.with_state(|state| (state.vk_windows.clone(), state.icons.clone()));
        let windows = vk_windows
            .lock()
            .unwrap()
            .iter()
            .filter_map(Weak::upgrade)
            .filter(|window| window.had_native_window())
            .collect::<Vec<_>>();
        for window in &windows {
            window.create_native_window(event_loop, window_registry, &icons.lock().unwrap());
        }
        windows
    }

    fn apply_teardown(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
        teardown: SurfaceTeardown,
    ) {
        match teardown {
            SurfaceTeardown::Ignored => {}
            SurfaceTeardown::SuspendAndDestroyWindows => {
                self.suspend_surfaces(event_loop, window_registry);
                self.wait_for_surface_teardown(window_registry);
                self.detach_native_windows(window_registry);
            }
        }
    }

    fn wait_for_surface_teardown(&self, window_registry: &WindowRegistry) {
        let (on_surface_teardown_timeout, surface_lifecycle) = self.with_state(|state| {
            (
                state.on_surface_teardown_timeout.clone(),
                state.surface_lifecycle.clone(),
            )
        });

        // Surfaces must be gone before their native windows are detached.
        let affected_window_ids = window_registry.surface_window_ids();
        if surface_lifecycle.wait_until_destroyed(&affected_window_ids, SURFACE_DESTROY_TIMEOUT) {
            return;
        }

        if let Some(on_surface_teardown_timeout) = on_surface_teardown_timeout {
            on_surface_teardown_timeout();
        }
        // Returning to Android with a live VkSurfaceKHR would let the OS
        // invalidate its native window while Vulkan still owns it.
        std::process::abort();
    }

    fn detach_native_windows(&self, window_registry: &mut WindowRegistry) {
        let vk_windows = self.with_state(|state| state.vk_windows.clone());
        for window in vk_windows.lock().unwrap().iter().filter_map(Weak::upgrade) {
            window.detach_native_window();
        }
        window_registry.clean();
    }

    fn resume_surfaces(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        self.get_mailbox().resume_surfaces();
        self.proxy_wake_up(event_loop, window_registry);
    }

    fn suspend_surfaces(
        &self,
        event_loop: &dyn ActiveEventLoop,
        window_registry: &mut WindowRegistry,
    ) {
        let mailbox = self.get_mailbox();
        mailbox.suspend_surfaces();

        // Complete operations queued just before suspension with their
        // unavailable/no-op result; do not leave the guest blocked while it
        // needs to process SurfaceLost.
        Self::run_requests(&mailbox.surface_requests, event_loop, window_registry);
        window_registry.notify_surface_lost();
    }

    fn notify_surfaces_attached(windows: impl IntoIterator<Item = Arc<WinitWindow>>) {
        for window in windows {
            window
                .event_sink
                .push_critical_event(Event::SurfaceAttached);
            window.event_sink.push_event(Event::RedrawRequested);
        }
    }

    pub fn simulate_destroy_surfaces(&self) {
        self.schedule_simulated_lifecycle_event(|state| {
            LifecycleAction::Teardown(state.simulated_lost())
        });
    }

    pub fn simulate_can_create_surfaces(&self) {
        self.schedule_simulated_lifecycle_event(|state| {
            LifecycleAction::Availability(state.simulated_available())
        });
    }

    fn schedule_simulated_lifecycle_event(
        &self,
        update: impl FnOnce(&mut LifecycleState) -> LifecycleAction + Send + 'static,
    ) {
        let weak_proxy = self.downgrade();
        self.get_mailbox()
            .execute(move |event_loop, window_registry| {
                let Some(proxy) = weak_proxy.upgrade() else {
                    return;
                };
                let action = proxy.update_lifecycle_state(update);
                proxy.apply_lifecycle_action(event_loop, window_registry, action);
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
        self.with_state(|state| {
            let simulate_touch = state.simulate_touch;
            if let Some(window) = window_registry.get_window_by_winit_id(window_id) {
                match &event {
                    WindowEvent::ThemeChanged(theme) => {
                        set_window_icon(
                            window.window.as_ref().as_ref(),
                            *theme,
                            &state.icons.lock().unwrap(),
                        );
                    }
                    _ => {}
                }

                window.on_event(event, &mut state.finger_id_map, simulate_touch);
            }
        });
    }

    pub(crate) fn get_mailbox(&self) -> Mailbox {
        self.with_state(|state| state.mailbox.clone())
    }
}

#[derive(Clone)]
pub struct WeakWinitProxy {
    state: Weak<Mutex<WinitProxyState>>,
}

impl WeakWinitProxy {
    pub fn upgrade(&self) -> Option<WinitProxy> {
        self.state.upgrade().map(|state| WinitProxy { state })
    }
}
