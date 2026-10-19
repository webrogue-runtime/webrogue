use std::sync::Arc;

use winit::window::Theme;

pub struct WindowIcons {
    pub light: Option<image::RgbaImage>,
    pub dark: Option<image::RgbaImage>,
}

pub fn set_window_icon(window: &dyn winit::window::Window, theme: Theme, icons: &WindowIcons) {
    let icon = match theme {
        winit::window::Theme::Light => icons.light.as_ref().or(icons.dark.as_ref()),
        winit::window::Theme::Dark => icons.dark.as_ref().or(icons.light.as_ref()),
    };
    window.set_window_icon(icon.and_then(|icon| {
        Some(winit::icon::Icon(Arc::new(
            winit::icon::RgbaIcon::new(icon.to_vec(), icon.width(), icon.height()).ok()?,
        )))
    }));
}
