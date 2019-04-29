use std::thread;
use std::sync::mpsc;

use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Mesh, Rect};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};
use nalgebra;

pub struct Renderer {
    handle: thread::JoinHandle<()>,
    sender: mpsc::Sender<f32>,
}

impl Renderer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(|| {
            let c = conf::Conf::new();
            let (ref mut ctx, ref mut event_loop) = &mut ContextBuilder::new("chip8", "Andreas Arvidsson")
                .conf(c)
                .build()
                .expect("Unable to create ggex context!");

            let mut game = MyGame::new(ctx, rx);

            match event::run(ctx, event_loop, &mut game) {
                Ok(_) => println!("Exited cleanly."),
                Err(e) => println!("Error occured: {}", e),
            }
        });

        Renderer {
            handle,
            sender: tx,
        }
    }

    pub fn render(&self, x: f32) {
        self.sender.send(x);
    }
}

struct MyGame {
    dt: std::time::Duration,
    pixel_mesh: Mesh,
    x: f32,
    receiver: mpsc::Receiver<f32>,
}

impl MyGame {
    fn new(ctx: &mut Context, receiver: mpsc::Receiver<f32>) -> MyGame {
        let mut rect = Rect::one();
        rect.scale(10.0, 10.0);
        let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::new(0.0, 1.0, 0.0, 1.0))
            .unwrap();

        MyGame {
            dt: std::time::Duration::new(0, 0),
            pixel_mesh: mesh,
            x: 13.0,
            receiver,
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.dt = timer::delta(ctx);
        if let Ok(x) = self.receiver.try_recv() {
            self.x = x;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        println!("delta: {}", self.dt.subsec_nanos());
        let my_dest = nalgebra::Point2::new(self.x, 37.0);
        graphics::clear(ctx, Color::new(1.0, 0.0, 0.0, 1.0));
        graphics::draw(ctx, &self.pixel_mesh, DrawParam::default().dest(my_dest)).unwrap();
        graphics::present(ctx).unwrap();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods,
        repeat: bool,
    ) {
        println!(
            "Key pressed: {:?}, modifier {:?}, repeat: {}",
            keycode, keymod, repeat
        );
    }
}
