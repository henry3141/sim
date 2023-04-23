use flo_canvas::Color;

use crate::ecs::*;

pub struct PHYS {
    pub gravity:f32,
    pub borders:(f32,f32,f32,f32)
}

struct Circle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    r: f32,
    color:Color,
}

impl Circle {
    fn intersects(&self, other: &Circle) -> Option<(f32, f32, [f32; 2])> {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dvx = other.vx - self.vx;
        let dvy = other.vy - self.vy;
        let dr = self.r + other.r;
        let a = dvx * dvx + dvy * dvy;
        let b = 2.0 * (dx * dvx + dy * dvy);
        let c = dx * dx + dy * dy - dr * dr;

        if a.abs() < 0.0001 {
            return None; // circles are moving parallel, no collision possible
        }

        let delta = b * b - 4.0 * a * c;

        if delta < 0.0 {
            return None; // no intersection
        }

        let t = (-b - delta.sqrt()) / (2.0 * a);
        if t < 0.0 || t > 1.0 {
            return None; // intersection is behind or beyond the movement
        }

        let x = self.x + self.vx * t;
        let y = self.y + self.vy * t;
        let distance = ((x - self.x).powi(2) + (y - self.y).powi(2)).sqrt();
        let normal = [dx + dvx * t, dy + dvy * t];
        let mag = (normal[0] * normal[0] + normal[1] * normal[1]).sqrt();

        Some((mag, distance, [normal[0] / mag, normal[1] / mag]))
    }

    fn to_bundle(&self) -> Bundle {
        vec![
            Component::Name("".to_string()),
            Component::Vec2(Component::Float(self.x).into(),Component::Float(self.y).into()),
            Component::Vec2(Component::Float(self.vx).into(),Component::Float(self.vy).into()),
            Component::GRAPHICS(Shape::Circle { radius: self.r, color: self.color, position: (self.x, self.y) }),
            Component::FUNC(Box::new(PHYS { gravity: 1.0, borders: (-500.0,-500.0,500.0,500.0) }), "PHYS".to_string()),
        ]
    }

    fn from_bundle(bundle: &Bundle) -> Circle {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut vx = 0.0;
        let mut vy = 0.0;
        let mut r = 0.0;
        let mut color2 = flo_canvas::Color::Rgba(0.0, 0.0, 0.0, 1.0);
        let mut pos = true;
        for i in bundle.iter() {
            match i {
                Component::Vec2(a,b) => {
                    match a.as_ref() {
                        Component::Float(f) => {
                            if pos {
                                x = *f;
                            } else {
                                vx = *f;
                            }
                        }
                        _ => {}
                    }
                    match b.as_ref() {
                        Component::Float(f) => {
                            if pos {
                                y = *f;
                                pos = false;
                            } else {
                                vy = *f;
                            }
                        }
                        _ => {}
                    }
                }
                Component::GRAPHICS(g) => {
                    match g {
                        Shape::Circle { radius, color, position } => {
                            r = *radius;
                            color2 = *color;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Circle { x, y, vx, vy, r , color:color2}
    }

    fn get_speed(&self) -> f32 {
        (self.vx.powi(2) + self.vy.powi(2)).sqrt()
    }

    fn collision<'a>(&mut self,ecs:&'a mut ECS) -> Option<((f32, f32, [f32; 2]),&'a mut Bundle)> {
        let mut entity = None;
        let mut data = None;
        //the len of (self.vx,self.vy)
        let len = self.get_speed() + (self.r * 2.0);
        for i in ecs.entitys.iter_mut() {
            if let Component::Name(i) = &i[0] {
                if i == "SELF" {
                    continue;
                }
            }
            let circle = Circle::from_bundle(&i);
            if self.distance(&circle) > len {
                continue;
            }
            if let Some((mag, distance, normal)) = self.intersects(&circle) {
                if data.is_none() {
                    data = Some((mag, distance, normal));
                    entity = Some(i);
                } else {
                    if distance < data.unwrap().1 {
                        data = Some((mag, distance, normal));
                        entity = Some(i);
                    }
                }
            }
        } 
        if data.is_none() {
            return None;
        }
        Some((data.unwrap(),entity.unwrap()))
    }

   
   
    fn reflect_velocities(circle1: &mut Circle, circle2: &mut Circle, normal: [f32; 2], loss: f32) {
        //reflect velcotites of normal while transferring about 0.6 of the velocities of each circle to the other
        let mut v1n = circle1.vx * normal[0] + circle1.vy * normal[1];
        let mut v2n = circle2.vx * normal[0] + circle2.vy * normal[1];
        let mut v1t = circle1.vx * normal[1] - circle1.vy * normal[0];
        let mut v2t = circle2.vx * normal[1] - circle2.vy * normal[0];
        let v1n2 = v2n;
        let v2n2 = v1n;
        v1n = v1n2 * loss;
        v2n = v2n2 * loss;
        circle1.vx = v1n * normal[0] - v1t * normal[1];
        circle1.vy = v1n * normal[1] + v1t * normal[0];
        circle2.vx = v2n * normal[0] - v2t * normal[1];
        circle2.vy = v2n * normal[1] + v2t * normal[0];
    }

    fn physics(&mut self,ecs:&mut ECS,phys:&PHYS) {
        //attract to center
        let mut dx = 0.0 - self.x;
        let mut dy = 0.0 - self.y;
        let mag = (dx * dx + dy * dy).sqrt();
        dx /= mag;
        dy /= mag;
        self.vx += dx * phys.gravity;
        self.vy += dy * phys.gravity;
        let hit = self.collision(ecs);
        if hit.is_some() {
            let ((mag, distance, normal),entity) = hit.unwrap();
            let mut circle = Circle::from_bundle(&entity);
            Circle::reflect_velocities(self,&mut circle,normal,0.8);
            *entity = circle.to_bundle();
        } else {
            self.x += self.vx;
            self.y += self.vy;
        }
        for i in 0..5 {self.move_away(ecs);}
    }

    fn collides(&self,other:&Circle) -> Option<(f32, f32, [f32; 2])> {
        let distance = ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt();
        let mag = distance - self.r - other.r;
        if mag < 0.0 {
            let normal = [(self.x - other.x) / distance, (self.y - other.y) / distance];
            return Some((mag, distance, normal));
        }
        None
    }

    fn distance(&self,other:&Circle) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn move_away(&mut self,ecs:&mut ECS) {
        //check if self is in another circle , and if so move the two away from each other
        //calculate the len of (self.vx,self.vy)
        for i in ecs.entitys.iter_mut() {
            if let Component::Name(i) = &i[0] {
                if i == "SELF" {
                    continue;
                }
            }
            let circle = Circle::from_bundle(&i);
            if let Some((mag, distance, normal)) = self.collides(&circle) {
                self.x += normal[0] * (distance - self.r * 2.0) * -1.0;
                self.y += normal[1] * (distance - self.r * 2.0) * -1.0;
            }
        }
    }
}

/*
vec![
    Name([any]),
    Vec2(x,y),
    Vec2(vx,vy),
]
*/

pub fn atom() -> Bundle {
    vec![
        Component::Name("".to_string()),
        Component::Vec2(Component::Float(rand::random::<f32>() * 10000.0 * {if rand::random::<f32>() > 0.5 {-1.0} else {1.0}}).into(),Component::Float(rand::random::<f32>() * 10000.0 * {if rand::random::<f32>() > 0.5 {-1.0} else {1.0}}).into()),
        Component::Vec2(Component::Float(rand::random::<f32>() * 20.0).into(),Component::Float(rand::random::<f32>() * 20.0).into()),
        Component::GRAPHICS(Shape::Circle { radius: 5.0, color: flo_canvas::Color::Rgba(rand::random(), rand::random(), rand::random(), 1.0), position: (0.0, 0.0) }),
        Component::FUNC(Box::new(PHYS { gravity: 1.0, borders: (-500.0,-500.0,500.0,500.0) }), "PHYS".to_string()),
    ]
}


impl FUNC for PHYS {
    fn call(&mut self, entity: &mut Bundle, ecs: &mut ECS) {
        self.borders = (ecs.size.0 / -2.0, ecs.size.1 / -2.0, ecs.size.0 / 2.0, ecs.size.1 / 2.0);
        entity[0] = Component::Name("SELF".to_string());
        let mut circle = Circle::from_bundle(entity);
        circle.physics(ecs,self);
        *entity = circle.to_bundle();
    }

    fn name(&self) -> String {
        "PHYS".to_string()
    }
}