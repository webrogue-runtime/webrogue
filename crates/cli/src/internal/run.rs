use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clap::Args;
#[cfg(feature = "internal-run")]
use webrogue_gfx::{AbstractSystem, Window};
use webrogue_gfx::{AbstractWindow, EventSink, System};

#[derive(Args, Debug)]
pub struct InternalRunCommand {
    /// Path to a WRAPP file or webrogue.json configuration.
    path: PathBuf,
    /// Isolated persistent-storage root for this run.
    #[arg(long, value_name = "PATH")]
    persistent_dir: PathBuf,
    /// Capture the last CPU-rendered frame as a binary PPM image.
    #[arg(long, value_name = "PATH")]
    capture_ppm: Option<PathBuf>,
    /// Headless window width in pixels.
    #[arg(long, default_value_t = 64)]
    window_width: u32,
    /// Headless window height in pixels.
    #[arg(long, default_value_t = 64)]
    window_height: u32,
}

impl InternalRunCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.window_width > 0 && self.window_height > 0,
            "window size must be nonzero"
        );
        let vfs = webrogue_vfs::VFS::build_from_path(&self.path)?;
        let last_frame = Arc::new(Mutex::new(None));
        let system = System::new(Arc::new(HeadlessSystem {
            width: self.window_width,
            height: self.window_height,
            last_frame: last_frame.clone(),
        }));
        let runtime = webrogue_wasmtime::Runtime::new(system, vfs, &self.persistent_dir);
        let mut runtime = runtime.jit();
        // Like the public CLI runner, let traps terminate this child process directly.
        unsafe {
            runtime.allow_panic();
        }
        webrogue_wasmtime::block_on_default_executor(runtime.run())?;

        if let Some(path) = &self.capture_ppm {
            let pixels = last_frame
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| anyhow::anyhow!("guest did not present a CPU-rendered frame"))?;
            write_ppm(&path, self.window_width, self.window_height, &pixels)?;
        }

        Ok(())
    }
}

struct HeadlessSystem {
    width: u32,
    height: u32,
    last_frame: Arc<Mutex<Option<Vec<u32>>>>,
}

impl AbstractSystem for HeadlessSystem {
    fn make_window(&self) -> Window {
        Window::new(Arc::new(HeadlessWindow {
            width: self.width,
            height: self.height,
            last_frame: self.last_frame.clone(),
            events: EventSink::new(),
        }))
    }

    fn pump(&self) {}

    fn get_virgl_context(&self) -> Option<Arc<Mutex<webrogue_virgl::ContextContainer>>> {
        None
    }
}

struct HeadlessWindow {
    width: u32,
    height: u32,
    last_frame: Arc<Mutex<Option<Vec<u32>>>>,
    events: EventSink,
}

impl AbstractWindow for HeadlessWindow {
    fn get_logical_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_physical_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_vk_id(&self) -> Option<u32> {
        None
    }

    fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<bool> {
        let expected_len = usize::try_from(self.width)
            .ok()
            .and_then(|width| {
                usize::try_from(self.height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .ok_or_else(|| anyhow::anyhow!("headless window pixel count overflow"))?;
        anyhow::ensure!(
            pixels.len() == expected_len,
            "expected {expected_len} pixels for {}x{} headless window, received {}",
            self.width,
            self.height,
            pixels.len(),
        );
        *self.last_frame.lock().unwrap() = Some(pixels.to_vec());
        Ok(true)
    }

    fn get_event_stream(&self) -> webrogue_gfx::EventStream {
        self.events.subscribe()
    }
}

fn write_ppm(
    path: &std::path::Path,
    width: u32,
    height: u32,
    pixels: &[u32],
) -> anyhow::Result<()> {
    let mut output = std::fs::File::create(path)?;
    use std::io::Write as _;
    write!(output, "P6\n{width} {height}\n255\n")?;
    for pixel in pixels {
        output.write_all(&[
            ((pixel >> 16) & 0xff) as u8,
            ((pixel >> 8) & 0xff) as u8,
            (pixel & 0xff) as u8,
        ])?;
    }
    Ok(())
}
