use std::{
    ffi::{CStr, CString},
    str::FromStr,
    sync::Mutex,
};

use anyhow::Context;
use ash::{
    vk::{Instance, InstanceCreateInfo},
    Entry,
};

pub fn load_vulkan_entry(required: bool) -> Option<Entry> {
    load_cached(required)
}

fn load_cached(required: bool) -> Option<Entry> {
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
            Err(error) => {
                if required {
                    #[cfg(not(windows))]
                    {
                        eprintln!(
                            r"
This application requires a Vulkan-compatible graphics driver to run.

Drivers tried:
{error}
                            "
                        )
                    }

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
No Vulkan-compatible graphics driver has been found.
To resolve this, try one of the following:

1. Update you GPU driver to the latest version. Visit you manufacturer website (NVIDIA, AMD, INTEL) for detailed instructions

2. Install OpenCL™, OpenGL®, and Vulkan® Compatibility Pack

3. If you are an application developer, bundle a fallback driver (vulkan_dzn.dll, vulkan_lvp.dll or vk_swiftshader.dll)

Drivers tried:
{error}
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

    loader_state.try_load(
        "System's driver",
        unsafe { Entry::load().map_err(|err| anyhow::anyhow!("{}", err)) }.and_then(check_entry),
    )?;

    #[cfg(feature = "static-vk")]
    {
        loader_state.try_load("Statically linked Vulkan", load_static())?;

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

    #[cfg(feature = "static-vk-icd")]
    {
        loader_state.try_load("Statically linked ICD", load_static_icd())?;

        fn load_static_icd() -> anyhow::Result<Entry> {
            extern "system" {
                fn vk_icdGetInstanceProcAddr(
                    instance: ash::vk::Instance,
                    name: *const std::ffi::c_char,
                ) -> ash::vk::PFN_vkVoidFunction;
            }

            let static_fn = ash::StaticFn::load_checked(move |name| unsafe {
                vk_icdGetInstanceProcAddr(ash::vk::Instance::null(), name.as_ptr())
                    .map(|f| f as *const std::ffi::c_void)
                    .unwrap_or(std::ptr::null())
            })?;
            check_entry(unsafe { Entry::from_static_fn(static_fn) })
        }
    }

    #[cfg(windows)]
    {
        loader_state.try_load("Dozen", (|| load_dynamic(&find_dll("vulkan_dzn.dll")?))())?;
        loader_state.try_load(
            "Lavapipe",
            (|| load_dynamic(&find_dll("vulkan_lvp.dll")?))(),
        )?;
        loader_state.try_load(
            "SwiftShader",
            (|| load_dynamic(&find_dll("vk_swiftshader.dll")?))(),
        )?;

        fn find_dll(filename: &str) -> anyhow::Result<std::path::PathBuf> {
            let path = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Path error"))?
                .join(filename);
            anyhow::ensure!(path.exists(), "{} not found", filename);
            Ok(path)
        }
    }

    Ok(())
}

#[cfg(any(windows, target_os = "macos"))]
fn load_dynamic(path: &std::path::PathBuf) -> anyhow::Result<Entry> {
    check_entry(unsafe { Entry::load_from(path) }?)
}

#[cfg(windows)]
fn load_dynamic_icd(path: &std::path::PathBuf) -> anyhow::Result<Entry> {
    use std::sync::Arc;
    let lib = Arc::new(unsafe { libloading::Library::new(path) }?);

    lazy_static::lazy_static! {
        static ref LATEST_ICD_LIB: Mutex<Option<Arc<libloading::Library>>> = Mutex::new(None);
    }
    *LATEST_ICD_LIB.lock().unwrap() = Some(lib.clone());

    let load_fn = Arc::new(unsafe {
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

    let instance_extensions = unsafe {
        entry
            .enumerate_instance_extension_properties(None)
            .context("Error while calling vkEnumerateInstanceExtensionProperties")?
    };
    let has_instance_extension = |name: &CStr| -> bool {
        for extension in &instance_extensions {
            if extension.extension_name_as_c_str().unwrap() == name {
                return true;
            }
        }
        return false;
    };
    let vk_version = unsafe {
        entry
            .try_enumerate_instance_version()
            .context("Error while calling vkEnumerateInstanceVersion")
    }?
    .unwrap_or(ash::vk::API_VERSION_1_0);
    anyhow::ensure!(
        (has_instance_extension)(ash::vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME)
            || vk_version >= ash::vk::API_VERSION_1_1,
        "{} extension is missing",
        ash::vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME
            .to_str()
            .unwrap()
    );
    anyhow::ensure!(
        (has_instance_extension)(ash::vk::KHR_EXTERNAL_MEMORY_CAPABILITIES_NAME)
            || vk_version >= ash::vk::API_VERSION_1_1,
        "{} extension is missing",
        ash::vk::KHR_EXTERNAL_MEMORY_CAPABILITIES_NAME
            .to_str()
            .unwrap()
    );
    anyhow::ensure!(
        (has_instance_extension)(ash::vk::KHR_EXTERNAL_SEMAPHORE_CAPABILITIES_NAME)
            || vk_version >= ash::vk::API_VERSION_1_1,
        "{} extension is missing",
        ash::vk::KHR_EXTERNAL_SEMAPHORE_CAPABILITIES_NAME
            .to_str()
            .unwrap()
    );
    anyhow::ensure!(
        (has_instance_extension)(ash::vk::KHR_EXTERNAL_FENCE_CAPABILITIES_NAME)
            || vk_version >= ash::vk::API_VERSION_1_1,
        "{} extension is missing",
        ash::vk::KHR_EXTERNAL_FENCE_CAPABILITIES_NAME
            .to_str()
            .unwrap()
    );

    let create_info = InstanceCreateInfo::default();
    let instance = unsafe { entry.create_instance(&create_info, None) }
        .context("Error while creating instance")?;
    // panic!(
    //     "version: {:?}",
    //     unsafe { entry.try_enumerate_instance_version() }.unwrap()
    // );
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
