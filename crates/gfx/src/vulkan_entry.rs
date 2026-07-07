use std::{
    ffi::CString,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use ash::{
    vk::{Instance, InstanceCreateInfo},
    Entry,
};

pub fn load_vulkan_entry(required: bool) -> Option<Entry> {
    load_cached(required)
}

pub fn load_cached(required: bool) -> Option<Entry> {
    lazy_static::lazy_static! {
        static ref CACHED_ENTRY: Mutex<Option<Entry>> = Mutex::new(None);
    }
    if let Some(entry) = CACHED_ENTRY.lock().unwrap().as_ref() {
        return Some(entry.clone());
    }
    let result = load_with_retry(required);
    if let Some(entry) = result.as_ref() {
        *CACHED_ENTRY.lock().unwrap() = Some(entry.clone());
    }
    result
}

fn load_with_retry(required: bool) -> Option<Entry> {
    loop {
        match load_parsed() {
            Ok((entry, _name)) => return Some(entry),
            Err(_error) => {
                if required {
                    #[cfg(windows)]
                    {
                        use windows::{
                            core::PCWSTR,
                            Win32::UI::WindowsAndMessaging::{
                                MessageBoxW, IDRETRY, MB_ICONERROR, MB_RETRYCANCEL, MB_TASKMODAL,
                            },
                        };

                        let mut title =
                            "Vulkan Driver Not Found".encode_utf16().collect::<Vec<_>>();
                        title.push(0);
                        let mut message = format!(
                            r"
This application requires a Vulkan-compatible graphics driver to run.
To resolve this, please update you GPU driver to the latest version.
Visit you manufacturer website (NVIDIA, AMD, INTEL) for detailed instructions.
If you are an application developer, you can bundle a fallback Vulkan driver (Lavapipe or SwiftShader).

Drivers tried:
{_error}
                    "
                        )
                        .trim()
                        .encode_utf16()
                        .collect::<Vec<_>>();

                        message.push(0);

                        let result = unsafe {
                            MessageBoxW(
                                None,
                                PCWSTR(message.as_ptr()),
                                PCWSTR(title.as_ptr()),
                                MB_RETRYCANCEL | MB_ICONERROR | MB_TASKMODAL,
                            )
                        };
                        if result == IDRETRY {
                            continue;
                        }
                    }
                }

                return None;
            }
        }
    }
}

fn load_parsed() -> Result<(Entry, &'static str), String> {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    std::env::set_var("MVK_CONFIG_LOG_LEVEL", "1");

    let mut loader_state = LoaderState::Loading(Vec::new());

    let _ = load_impl(&mut loader_state);

    match loader_state {
        LoaderState::Loading(errors) => {
            let mut err = "".to_string();
            for (name, message) in errors {
                err += &format!("{}: {:#}\n\n", name, message);
            }
            return Err(err.trim().to_string());
        }
        LoaderState::Loaded((entry, name)) => Ok((entry, name)),
    }
}

fn load_impl(loader_state: &mut LoaderState) -> Result<(), ()> {
    #[cfg(target_os = "macos")]
    {
        loader_state.try_load("libMoltenVK.dylib", load_dynamic_moltenvk())?;

        fn load_dynamic_moltenvk() -> anyhow::Result<Entry> {
            use std::env::current_exe;

            let mut path = current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("libMoltenVK.dylib");
            if !path.exists() {
                path = current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("Resources")
                    .join("libMoltenVK.dylib");
            };
            if !path.exists() {
                anyhow::bail!("libMoltenVK.dylib not found")
            }
            load_dynamic(&path)
        }
    }

    loader_state.try_load("System's implementation", unsafe {
        Entry::load().map_err(|err| anyhow::anyhow!("{}", err))
    })?;

    #[cfg(feature = "static-vk")]
    {
        loader_state.try_load("Statically linked implementation", load_static())?;

        fn load_static() -> anyhow::Result<Entry> {
            extern "system" {
                fn vkGetInstanceProcAddr(
                    instance: ash::vk::Instance,
                    name: *const std::ffi::c_char,
                ) -> ash::vk::PFN_vkVoidFunction;
            }

            return check_entry(unsafe {
                Entry::from_static_fn(ash::StaticFn {
                    get_instance_proc_addr: vkGetInstanceProcAddr,
                })
            });
        }
    }

    #[cfg(windows)]
    {
        loader_state.try_load("Dozen", load_dynamic_dozen())?;

        fn load_dynamic_dozen() -> anyhow::Result<Entry> {
            let path = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path error"))?
                .join("vulkan_dzn.dll");

            load_dynamic_icd(&path)
        }
    }

    #[cfg(windows)]
    {
        loader_state.try_load("Lavapipe", load_dynamic_lavapipe())?;

        fn load_dynamic_lavapipe() -> anyhow::Result<Entry> {
            let path = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path error"))?
                .join("vulkan_lvp.dll");

            load_dynamic_icd(&path)
        }
    }

    #[cfg(windows)]
    {
        loader_state.try_load("SwiftShader", load_dynamic_swiftshader())?;

        fn load_dynamic_swiftshader() -> anyhow::Result<Entry> {
            let path = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path error"))?
                .join("vk_swiftshader.dll");

            load_dynamic(&path)
        }
    }

    Ok(())
}

fn load_dynamic(path: &PathBuf) -> anyhow::Result<Entry> {
    check_entry(unsafe { Entry::load_from(path) }?)
}

fn load_dynamic_icd(path: &PathBuf) -> anyhow::Result<Entry> {
    let lib = std::sync::Arc::new(unsafe { libloading::Library::new(path) }?);

    lazy_static::lazy_static! {
        static ref LATEST_ICD_LIB: Mutex<Option<Arc<libloading::Library>>> = Mutex::new(None);
    }
    *LATEST_ICD_LIB.lock().unwrap() = Some(lib.clone());

    let load_fn = std::sync::Arc::new(unsafe {
        lib.get::<ash::vk::PFN_vkGetInstanceProcAddr>(b"vk_icdGetInstanceProcAddr")?
    });
    let static_fn = ash::StaticFn::load_checked(move |name| unsafe {
        (load_fn)(ash::vk::Instance::null(), name.as_ptr())
            .map(|f| f as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null())
    })?;
    check_entry(unsafe { Entry::from_static_fn(static_fn) })
}

fn check_entry(entry: Entry) -> anyhow::Result<Entry> {
    if unsafe {
        (entry.static_fn().get_instance_proc_addr)(
            Instance::null(),
            CString::from_str("vkCreateInstance").unwrap().as_ptr(),
        )
        .is_none()
    } {
        return Err(anyhow::anyhow!(
            "vkGetInstanceProcAddr(NULL, \"vkCreateInstance\") returns NULL"
        ));
    }

    let create_info = InstanceCreateInfo::default();
    let instance = unsafe { entry.create_instance(&create_info, None) }
        .context("Error while creating instance")?;
    let instance2 = instance.clone();
    let instance_drop_callback = DropCallback::new(Box::new(move || unsafe {
        instance2.destroy_instance(None)
    }));
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .context("Error while enumeration physical devices")?
    };
    anyhow::ensure!(
        !physical_devices.is_empty(),
        "No physical devices available"
    );
    drop(instance_drop_callback);

    return Ok(entry);
}

enum LoaderState {
    Loading(Vec<(&'static str, anyhow::Error)>),
    Loaded((Entry, &'static str)),
}

impl LoaderState {
    fn try_load(&mut self, name: &'static str, entry: anyhow::Result<Entry>) -> Result<(), ()> {
        let LoaderState::Loading(errors) = self else {
            return Err(());
        };
        match entry {
            Ok(entry) => {
                *self = LoaderState::Loaded((entry, name));
                return Err(());
            }
            Err(error) => {
                errors.push((name, error));
                return Ok(());
            }
        }
    }
}

pub struct DropCallback(Option<Box<dyn FnOnce() + Send>>);

impl DropCallback {
    pub fn new(f: Box<dyn FnOnce() + Send>) -> Self {
        Self(Some(f))
    }
}

impl Drop for DropCallback {
    fn drop(&mut self) {
        (self.0.take().unwrap())();
    }
}
