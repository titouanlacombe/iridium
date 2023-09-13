pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self::new(r, g, b, a)
    }

    pub fn to_rgba(&self) -> (f64, f64, f64, f64) {
        (self.r, self.g, self.b, self.a)
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xff) as f64 / 255.0;
        let g = ((hex >> 16) & 0xff) as f64 / 255.0;
        let b = ((hex >> 8) & 0xff) as f64 / 255.0;
        let a = (hex & 0xff) as f64 / 255.0;
        Self { r, g, b, a }
    }

    pub fn to_hex(&self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        let a = (self.a * 255.0) as u32;
        (r << 24) | (g << 16) | (b << 8) | a
    }

    pub fn from_hsva(h: f64, s: f64, v: f64, a: f64) -> Self {
        let c = v * s;
        let h = h / 60.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            5 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };
        let m = v - c;
        Self {
            r: r + m,
            g: g + m,
            b: b + m,
            a,
        }
    }

    pub fn to_hsva(&self) -> (f64, f64, f64, f64) {
        let c_max = self.r.max(self.g).max(self.b);
        let c_min = self.r.min(self.g).min(self.b);
        let delta = c_max - c_min;
        let h = if delta == 0.0 {
            0.0
        } else if c_max == self.r {
            60.0 * (((self.g - self.b) / delta) % 6.0)
        } else if c_max == self.g {
            60.0 * ((self.b - self.r) / delta + 2.0)
        } else {
            60.0 * ((self.r - self.g) / delta + 4.0)
        };
        let s = if c_max == 0.0 { 0.0 } else { delta / c_max };
        let v = c_max;
        (h, s, v, self.a)
    }
}
