use sfml::{
    cpp::FBox,
    graphics::{FloatRect, RenderWindow, View},
    system::Vector2,
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

    pub fn make(&self) -> FBox<RenderWindow> {
        let mut window =
            RenderWindow::new(self.size, self.title.as_str(), self.style, &self.settings)
                .expect("Window creation failed");
        window.set_key_repeat_enabled(self.key_repeat_enabled);
        window
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

    pub fn make(&self) -> FBox<View> {
        let mut view = View::new().expect("Failed to create view");
        view.set_center(self.center);
        view.set_size(self.size);
        view.set_viewport(self.viewport);
        view.set_rotation(self.rotation);
        view.zoom(self.zoom);
        view
    }
}
