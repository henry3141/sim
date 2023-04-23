use crate::sys::*;
use flo_canvas::Color;
use flo_canvas::{GraphicsContext, GraphicsPrimitives};
use flo_draw::Key;

pub struct Default {
    pub height: f32,
    pub width: f32,
    pub cam: (f32, f32),
    pub zoom: f32,
}

impl Default {
    pub fn new() -> Default {
        Default {
            height: 800.0,
            cam: (0.0, 0.0),
            width: 1200.0,
            zoom: 1.0,
        }
    }
}

fn float_(multi: &Multi) -> f32 {
    match multi {
        Multi::Float(x) => *x,
        _ => 0.0,
    }
}

fn vec_(multi: &Multi) -> Vec<f32> {
    match multi {
        Multi::Vec(x) => x.iter().map(|x| float_(x)).collect(),
        _ => vec![],
    }
}

impl SystemGRAPHICS for Default {
    //fix weird positioning
    fn update(
        &mut self,
        entitys: Vec<&mut Entity>,
        api: &mut API,
        draw: &mut flo_canvas::DrawingTarget,
    ) {
        let mut positions = vec![];
        entitys
            .iter()
            .map(|x| (x.position, x.data.get("color").unwrap()))
            .for_each(|(pos, color)| {
                let color = vec_(color);
                let color = Color::Rgba(color[0], color[1], color[2], 1.0);
                positions.push((pos.0, pos.1, color));
            });
        draw.draw(|gc| {
            //set canvas size
            gc.canvas_height(self.height);
            gc.center_region(
                (-(self.width / 2.0)) + self.cam.0 * self.zoom,
                (-(self.height / 2.0)) + self.cam.1 * self.zoom,
                (self.width / 2.0) + self.cam.0 * self.zoom,
                (self.height / 2.0) + self.cam.1 * self.zoom,
            );
            gc.clear_canvas(Color::Rgba(0.0, 0.0, 0.0, 1.0));
            for i in &positions {
                gc.new_path();
                gc.circle(
                    i.0 / self.width,
                    i.1 / self.height,
                    10.0 / self.width / self.zoom,
                );
                gc.fill_color(i.2);
                gc.fill();
            }
        });
        if api.pressed(Key::KeyS) {
            self.cam = (self.cam.0, self.cam.1 - 10.0);
        }
        if api.pressed(Key::KeyW) {
            self.cam = (self.cam.0, self.cam.1 + 10.0);
        }
        if api.pressed(Key::KeyA) {
            self.cam = (self.cam.0 - 10.0, self.cam.1);
        }
        if api.pressed(Key::KeyD) {
            self.cam = (self.cam.0 + 10.0, self.cam.1);
        }
        if api.pressed(Key::KeySpace) {
            self.zoom += 1.0;
            if self.zoom > 10.0 {
                self.zoom = 10.0;
            }
        }
        if api.pressed(Key::KeyE) {
            //find median position
            let mut x = 0.0;
            let mut y = 0.0;
            for i in &positions {
                x += i.0;
                y += i.1;
            }
            x /= positions.len() as f32;
            y /= positions.len() as f32;
            self.cam = (x, y);
        }
        if api.pressed(Key::KeyQ) {
            self.zoom -= 1.0;
            if self.zoom < 1.0 {
                self.zoom = 1.0;
            }
        }
        if api.pressed(Key::KeyY) {
            self.cam = (0.0, 0.0);
            self.zoom = 1.0;
        }
    }

    fn tags(&self) -> Vec<String> {
        vec!["draw".to_string()]
    }
}
