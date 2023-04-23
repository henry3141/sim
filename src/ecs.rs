use flo_canvas::{Color, GraphicsContext, GraphicsPrimitives};
use flo_draw::{create_drawing_window_with_events, with_2d_graphics, DrawEvent};
use futures::{executor, StreamExt};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

pub struct Waiter {
    pub time: std::time::Duration,
    pub start: std::time::Instant,
}

impl Waiter {
    pub fn new(time: std::time::Duration) -> Waiter {
        Waiter {
            time,
            start: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self) -> bool {
        if self.start.elapsed() > self.time {
            self.start = std::time::Instant::now();
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.start = std::time::Instant::now();
    }

    pub fn time(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    pub fn wait(&mut self) {
        while self.start.elapsed() < self.time {}
    }
}

pub enum Component {
    GRAPHICS(Shape),
    FUNC(Box<dyn FUNC>,String),
    API(Box<dyn API>, String),
    String(String),
    Float(f32),
    Vec2(Box<Component>,Box<Component>),
    Vec(Vec<Component>),
    Bool(bool),
    Int(i32),
    Map(HashMap<String, Component>),
    Name(String),
}

pub enum ComponentType {
    GRAPHICS(Shape),
    FUNC(String),
    API(String),
    String,
    Float,
    Vec2(Box<ComponentType>,Box<ComponentType>),
    Vec(Vec<ComponentType>),
    Bool,
    Int,
    Map(HashMap<String,ComponentType>),
    Name(String),
    None,
}

//impl eq for Component and ComponentType

impl PartialEq<ComponentType> for Component {
    fn eq(&self, other: &ComponentType) -> bool {
        match (self, other) {
            (Component::GRAPHICS(i), ComponentType::GRAPHICS(s)) => i == s,
            (Component::API(_, m), ComponentType::API(n)) => m == n,
            (Component::String(_), ComponentType::String) => true,
            (Component::Float(_), ComponentType::Float) => true,
            (Component::Vec2(n1, n2), ComponentType::Vec2(n3, n4)) => n1.as_ref() == n3.as_ref() && n2.as_ref() == n4.as_ref(),
            (Component::Vec(i), ComponentType::Vec(n)) => i == n,
            (Component::Bool(_), ComponentType::Bool) => true,
            (Component::Int(_), ComponentType::Int) => true,
            (Component::Map(i), ComponentType::Map(i2)) => return {for (k,v) in i.iter() {
                if !(*v == *(i2.get(k).unwrap_or_else(|| {return &ComponentType::None}))) {
                    return false;
                }
            };true},
            (Component::Name(s), ComponentType::Name(s2)) => s == s2,
            _ => false,
        }
    }
}


#[derive(Clone, Debug,PartialEq)]
pub enum Shape {
    Circle {
        radius: f32,
        color: Color,
        position: (f32, f32),
    },
}

pub trait FUNC: Send {
    fn call(&mut self, entity: &mut Bundle, api: &mut ECS);
    fn name(&self) -> String;
}

pub trait API: Send {
    fn call(&mut self, api: &mut ECS, input: Vec<Component>) -> Component;
    fn name(&self) -> String;
}

pub trait System: Send {
    fn update(&mut self, entitys: Vec<&mut Bundle>, api: &mut ECS);
    fn name(&self) -> String;
    fn contains(&self) -> Vec<ComponentType>;
}

pub type Bundle = Vec<Component>;

pub struct ECS {
    pub entitys: Vec<Bundle>,
    pub systems: Vec<Box<dyn System>>,
    pub draw: flo_canvas::DrawingTarget,
    pub events: Ref<Vec<DrawEvent>>,
    pub size: (f32, f32),
}

pub struct Builder {
    pub entitys: Vec<Bundle>,
    pub systems: Vec<Box<dyn System>>,
}

impl Builder {
    pub fn add(self, system: Box<dyn System>) -> Builder {
        let mut systems = self.systems;
        systems.push(system);
        Builder { systems, ..self }
    }

    #[inline]
    pub fn run(self) {
        let builder = self;
        with_2d_graphics(|| {
            let (canvas, events) = create_drawing_window_with_events("Physics");
            let vec = Ref::new(Vec::new());
            let mut c = vec.clone();
            std::thread::spawn(move || {
                executor::block_on(async {
                    let mut events = events;
                    while let Some(event) = events.next().await {
                        c.push(event);
                    }
                });
            });
            let mut ecs = Ref::new(ECS {
                entitys: builder.entitys,
                systems: builder.systems,
                draw: canvas,
                events: vec,
                size: (1200.0, 800.0),
            });
            let mut timer = Waiter::new(std::time::Duration::from_millis(1000 / 30));
            loop {
                timer.reset();
                for i in ecs.clone().get_mut().systems.iter_mut() {
                    i.update(
                        ecs.clone().get_mut().with(i.contains()),
                        ecs.get_mut(),
                    );
                }
                let mut graphics = vec![];
                for i in ecs.clone().get_mut().entitys.iter_mut() {
                    let mut ref_ = Ref::new(i);
                    for j in ref_.clone().get_mut().iter_mut() {
                        match j {
                            Component::GRAPHICS(g) => {
                                graphics.push(g.clone());
                            }
                            Component::FUNC(f,s) => {
                                f.call(ref_.get_mut(), ecs.get_mut());
                            }
                            _ => {}
                        }
                    }
                }
                ecs.clone().get_mut().draw.draw(|gc| {
                    gc.canvas_height(ecs.size.1);
                    gc.center_region(
                        ecs.size.0 / -2.0,
                        ecs.size.1 / -2.0,
                        ecs.size.0 / 2.0,
                        ecs.size.1 / 2.0,
                    );
                    gc.clear_canvas(Color::Rgba(0.0, 0.0, 0.0, 1.0));
                    for i in graphics.iter() {
                        match i {
                            Shape::Circle {
                                radius,
                                color,
                                position,
                            } => {
                                gc.new_path();
                                gc.circle(
                                    position.0 / ecs.size.0,
                                    position.1 / ecs.size.1,
                                    *radius / ecs.size.0,
                                );
                                gc.fill_color(*color);
                                gc.fill();
                            }
                        }
                    }
                });
                let took = timer.time();
                //calculate max fps at current delta time
                let max_fps = 1.0 / took.as_secs_f64();
                if max_fps < 60.0 {
                    println!("FPS: {}", max_fps);
                }
                timer.wait();
            }
        });
    }
}

impl ECS {
    pub fn start() -> Builder {
        Builder {
            entitys: vec![],
            systems: vec![],
        }
    }

    pub fn add_entity(&mut self, entity: Bundle) {
        self.entitys.push(entity);
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn with(&mut self,vec:Vec<ComponentType>) -> Vec<&mut Bundle> {
        self.entitys.iter_mut().filter(|x| {
            for (i, j) in x.iter().zip(vec.iter()) {
                if i != j {
                    return false;
                }
            }
            true
        }).collect()
    }

    pub fn remove_entity(&mut self, name: String) {
        self.entitys.retain(|x| {
            for i in x.iter() {
                match i {
                    Component::Name(n) => {
                        if n == &name {
                            return false;
                        }
                    }
                    _ => {}
                }
            }
            true
        });
    }

    pub fn remove_system(&mut self, name: String) {
        self.systems.retain(|x| {
            if x.name() == name {
                return false;
            }
            true
        });
    }
}

unsafe impl Sync for ECS {}

unsafe impl Send for ECS {}

unsafe impl<T> Send for Ref<T> {}

unsafe impl<T> Sync for Ref<T> {}

pub struct Ref<T> {
    pub ptr: *mut T,
    pub rc: usize,
}

impl<T> Ref<T> {
    pub fn new(t: T) -> Ref<T> {
        Ref {
            ptr: Box::into_raw(Box::new(t)),
            rc: 1,
        }
    }

    pub fn clone(&self) -> Ref<T> {
        Ref {
            ptr: self.ptr,
            rc: self.rc + 1,
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Ref<T> {
        Ref {
            ptr: self.ptr,
            rc: self.rc + 1,
        }
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Ref<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}
