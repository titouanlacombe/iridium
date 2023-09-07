use sfml::graphics::RenderWindow;

// Data without implementation to be sent across threads
pub struct WindowData {
    pub size: (u32, u32),
    pub title: String,
    pub style: sfml::window::Style,
    pub settings: sfml::window::ContextSettings,
}

impl WindowData {
    pub fn new(
        size: (u32, u32),
        title: String,
        style: sfml::window::Style,
        settings: sfml::window::ContextSettings,
    ) -> Self {
        Self {
            size,
            title,
            style,
            settings,
        }
    }

    pub fn create_real(&self) -> RenderWindow {
        RenderWindow::new(self.size, self.title.as_str(), self.style, &self.settings)
    }
}
