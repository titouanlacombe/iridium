use sfml::{
    graphics::{FloatRect, RenderWindow},
    system::Vector2,
    SfBox,
};

// Data without implementation to be sent across threads
pub struct WindowData {
    size: (u32, u32),
    title: String,
    style: sfml::window::Style,
    settings: sfml::window::ContextSettings,
    key_repeat_enabled: bool,
}

impl WindowData {
    pub fn new(
        size: (u32, u32),
        title: String,
        style: sfml::window::Style,
        settings: sfml::window::ContextSettings,
        key_repeat_enabled: bool,
    ) -> Self {
        Self {
            size,
            title,
            style,
            settings,
            key_repeat_enabled,
        }
    }

    pub fn make(&self) -> RenderWindow {
        let mut obj = RenderWindow::new(self.size, self.title.as_str(), self.style, &self.settings);
        obj.set_key_repeat_enabled(self.key_repeat_enabled);
        obj
    }
}

pub struct ViewData {
    pub center: Vector2<f32>,
    pub size: Vector2<f32>,
    pub viewport: FloatRect,
    pub rotation: f32,
    pub zoom: f32,
}

impl ViewData {
    pub fn new(
        center: Vector2<f32>,
        size: Vector2<f32>,
        viewport: FloatRect,
        rotation: f32,
        zoom: f32,
    ) -> Self {
        Self {
            center,
            size,
            viewport,
            rotation,
            zoom,
        }
    }

    pub fn make(&self) -> SfBox<sfml::graphics::View> {
        let mut view = sfml::graphics::View::new(self.center, self.size);
        view.set_viewport(self.viewport);
        view.set_rotation(self.rotation);
        view.set_size(self.size);
        view.zoom(self.zoom);
        view
    }
}
