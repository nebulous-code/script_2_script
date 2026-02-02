use raylib::prelude::*;

pub struct BounceEvent {
    pub bounced: bool,
    pub x: bool,
    pub y: bool,
}

pub struct BouncingBall {
    pub position: Vector2,
    pub velocity: Vector2,
    pub radius: f32,
    width: f32,
    height: f32,
}

impl BouncingBall {
    pub fn new(width: f32, height: f32) -> Self {
        let radius = height / 8.0;
        let position = Vector2::new(width / 2.0, height / 2.0);
        let velocity = Vector2::new(200.0, 200.0);
        Self {
            position,
            velocity,
            radius,
            width,
            height,
        }
    }

    pub fn step(&mut self, dt: f32) -> BounceEvent {
        let mut ev = BounceEvent {
            bounced: false,
            x: false,
            y: false,
        };

        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;

        if self.position.x - self.radius < 0.0 {
            self.position.x = self.radius;
            self.velocity.x = self.velocity.x.abs();
            ev.bounced = true;
            ev.x = true;
        } else if self.position.x + self.radius > self.width {
            self.position.x = self.width - self.radius;
            self.velocity.x = -self.velocity.x.abs();
            ev.bounced = true;
            ev.x = true;
        }

        if self.position.y - self.radius < 0.0 {
            self.position.y = self.radius;
            self.velocity.y = self.velocity.y.abs();
            ev.bounced = true;
            ev.y = true;
        } else if self.position.y + self.radius > self.height {
            self.position.y = self.height - self.radius;
            self.velocity.y = -self.velocity.y.abs();
            ev.bounced = true;
            ev.y = true;
        }

        ev
    }

    pub fn color_at(&self, t_norm: f32) -> Color {
        let hue = t_norm.rem_euclid(1.0);
        hsv_to_rgb(hue, 1.0, 1.0)
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    Color::new(to_u8(r), to_u8(g), to_u8(b), 255)
}

fn to_u8(v: f32) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}
