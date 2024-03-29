use ggez::*;
use ggez::input::mouse::MouseButton;
use ggez::input::keyboard::KeyCode;
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
struct Ball {
    x: f32,
    y: f32,
    xv: f32,
    yv: f32,
    stationary: bool,
    mass: f32,
    #[serde(skip)]
    trail: VecDeque<(f32,f32)>,
}

impl Ball {
    fn new(x: f32, y: f32) -> Ball {
        Ball{ x: x, y: y, xv: 0.0, yv: 0.0, stationary: false, mass: 10.0, trail: VecDeque::with_capacity(50)}
    }
    fn distance_from(&self, other: &Ball) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

struct Game {
    circle_vec: Vec<Ball>,
    stationary: bool,
    xv: f32,
    yv: f32,
    mass: f32,
    show_help: bool,
    draw_trails: bool,
    running: bool,
    sliding: bool,
    mouse_x: f32,
    mouse_y: f32,
    dest: mint::Point2<f32>,
    scale: mint::Vector2<f32>,
}

impl Game {
    fn new(_ctx: &mut Context) -> Game {
        Game{ circle_vec: Vec::new(), stationary: false, xv: 0.0, yv: 0.0, mass: 10.0, show_help: false, draw_trails: false,
             running: true, sliding: false, mouse_x: 0.0, mouse_y: 0.0, dest: mint::Point2{ x: 0.0, y: 0.0}, scale: mint::Vector2{ x: 1.0, y: 1.0}  }
    }
}

impl ggez::event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, 60) {
            if self.running {
                for i in 0..self.circle_vec.len() {
                    if self.circle_vec[i].stationary { continue; }
                    for j in 0..self.circle_vec.len() {
                        if i == j {continue;}
                        
                        // TODO: Set some gravity
                        // TODO: Incorporate Mass
                        // θ = tan-1 ( y / x )
                        // x = r × cos( θ )
                        // y = r × sin( θ )
                        // G: In SI units its value is approximately 6.674×10−11 m3⋅kg−1⋅s−2
                        // F = G * ( (m1 * m2) / r^2)

                        let distance_mod = 0.006674 * ((self.circle_vec[j].mass * self.circle_vec[i].mass) / self.circle_vec[i].distance_from(&self.circle_vec[j])) / self.circle_vec[i].mass;
                        let angle = (self.circle_vec[j].y - self.circle_vec[i].y).atan2(self.circle_vec[j].x - self.circle_vec[i].x);

                        self.circle_vec[i].xv += distance_mod * angle.cos();
                        self.circle_vec[i].yv += distance_mod * angle.sin();
                    }
                }
                for circle in self.circle_vec.iter_mut() {
                    if !circle.stationary {
                        if circle.trail.len() == circle.trail.capacity() {
                            let _ = circle.trail.pop_back();
                        }
                        circle.trail.push_front((circle.x,circle.y));
                        circle.x += circle.xv;
                        circle.y += circle.yv;
                    }
                }
            }
        }
        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::BLACK);

        draw_text(ctx, "Press F1 for help".to_string(), 128.0, 0.0);
        if self.show_help {
            draw_text(ctx,"Left click to place node\nLeft Arrow/Right Arrow to modify X velocity\
            \nUp Arrow/Down Arrow to modify Y velocity\nLeft Shift/Right Shift to modify mass of node\n\
            Enter/Return to reset velocity and mass to default values\nTab to pause simulation\nSpace to toggle \"Static\" mode\n\
            Backspace to clear all nodes\nF2 to toggle trails (performance heavy)\nEscape to exit\n\
            L to load a json file\nS to save the current state to a json file".to_string(), 128.0, 16.0);
        }

        if self.xv != 0.0 {
            draw_text(ctx,format!("xv: {}", self.xv),0.0,0.0);
        }
        if self.yv != 0.0 {
            draw_text(ctx,format!("yv: {}", self.yv),0.0,16.0);
        }
        if self.stationary {
            draw_text(ctx,"Placing Static Node".to_string(),0.0,32.0);
        }
        if self.mass != 0.0 {
            draw_text(ctx,format!("Mass: {}", self.mass),0.0,48.0);
        }
        if !self.running {
            draw_text(ctx,"Paused".to_string(),0.0,64.0);
        }


        graphics::draw_queued_text(ctx, graphics::DrawParam::default(), None, graphics::FilterMode::Linear)?;

        let matrix = graphics::DrawParam::new().dest(self.dest).scale(self.scale);

        for circle in self.circle_vec.iter() {
            let capacity = circle.trail.capacity() as f32;
            let mass = circle.mass.log10();
            let color = graphics::Color::new(((circle.xv.abs().hypot(circle.yv.abs())) / 10.0).min(1.0), 0.2, 0.2, 1.0);
            if self.draw_trails {
                let main_trail = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    mint::Point2{x:0.0, y:0.0},
                    10.0 * mass,
                    0.1,
                    color,
                )?;
                let mut trail_batch = graphics::MeshBatch::new(main_trail)?;

                for (id, trail) in circle.trail.iter().enumerate() {
                    let trail_modifier = 0.9 - ((id + 1) as f32 / capacity) as f32;
                    if trail_modifier < 0.0 { break; } // TODO: check if this line even fires

                    let param = graphics::DrawParam::default().dest(mint::Point2{x: trail.0, y: trail.1}).scale(mint::Vector2{x:trail_modifier, y: trail_modifier});
                    trail_batch.add(param);

                }

                trail_batch.draw(ctx, matrix);
            }
            let circle_blit = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                mint::Point2{x:circle.x  , y:circle.y},
                10.0 * mass ,
                0.1,
                color,
            )?;
            graphics::draw(ctx, &circle_blit, matrix)?;
        }
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        if (self.mouse_x - x).abs() < 0.1 && (self.mouse_y - y).abs() < 0.1 { return } // don't do anything if the mouse seems to have not moved
        self.mouse_x = x;
        self.mouse_y = y;
        if self.sliding {
            self.dest.x += dx;
            self.dest.y += dy;
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32)
    {
        self.scale.x += (self.scale.x * y) / 100.0;
        self.scale.y += (self.scale.y * y) / 100.0;
        if y > 0.0 {   
            self.dest.x -= self.mouse_x / 10.0;
            self.dest.y -= self.mouse_y / 10.0;
        } else {
            self.dest.x += self.mouse_x / 10.0;
            self.dest.y += self.mouse_y / 10.0;
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == ggez::input::mouse::MouseButton::Left {
            let mut ball = Ball::new((x - self.dest.x ) / self.scale.x, (y - self.dest.y ) / self.scale.y);
            ball.stationary = self.stationary;
            ball.xv = self.xv;
            ball.yv = self.yv;
            ball.mass = self.mass;
            self.circle_vec.push(ball);
        }
        if button == ggez::input::mouse::MouseButton::Right {
            self.sliding = true;
        }
    }

    fn mouse_button_up_event(&mut self,_ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        if button == ggez::input::mouse::MouseButton::Right {
            self.sliding = false;
        }
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: ggez::input::keyboard::KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape => {ctx.continuing = false;} // Ugly, might break, whatever
            KeyCode::Space => {self.stationary = !self.stationary;}
            KeyCode::Up => {self.yv += 0.1;}
            KeyCode::Down => {self.yv -= 0.1;}
            KeyCode::Right => {self.xv += 0.1;}
            KeyCode::Left => {self.xv -= 0.1;}
            KeyCode::Back => {self.circle_vec.clear();}
            KeyCode::RShift => {self.mass += 10.0;}
            KeyCode::LShift => {self.mass -= 10.0;}
            KeyCode::Return => {self.mass = 10.0; self.yv = 0.0; self.xv = 0.0;}
            KeyCode::F1 => {self.show_help = !self.show_help;}
            KeyCode::F2 => {self.draw_trails = !self.draw_trails;}
            KeyCode::Tab => {self.running = !self.running;}
            KeyCode::S => {save_to_file(&self.circle_vec)}
            KeyCode::L => {match load_from_file() {
                    Ok(sim) => {self.circle_vec = sim;self.running = false; for i in 0..self.circle_vec.len() {self.circle_vec[i].trail = VecDeque::with_capacity(50)}}
                    Err(e) => {eprintln!("Failure to load: {}", e)}
                }}
            _ => ()
        }
    }
}

fn draw_text(ctx: &mut Context, str: String, x: f32, y: f32) {
    graphics::queue_text(ctx, &graphics::Text::new(str),mint::Point2{x:x,y:y},Some(graphics::DrawParam::default().color));
}

fn save_to_file(circle_vec: &Vec<Ball>) {
    match nfd::open_save_dialog(None, None) {
        Ok(r)   => {
            match r {
                nfd::Response::Okay(file_path) => {
                    match std::fs::write(file_path, serde_json::to_string(circle_vec).unwrap()) {
                        Ok(_) => (),
                        Err(e) => {eprintln!("Error while writing: {}", e);}
                    }}
                _   =>  () // If they hit cancel, I don't care, and they shouldn't be able to select multiple files
            }
        }
        Err(e) => {eprintln!("Error occured while selecting save location: {}",e)}
    }
}

fn load_from_file() -> Result<Vec<Ball>, Box<dyn std::error::Error>> {
    let dialog = nfd::open_file_dialog(None, None)?;
    match dialog {
        nfd::Response::Okay(file_path) => load_from_json(&file_path),
        nfd::Response::Cancel   =>  load_from_json(&"".to_string()), // loads from blank file to create an error :shrug:
        nfd::Response::OkayMultiple(_x) =>  panic!("I'm pretty sure this can't happen"),
    }
}

fn load_from_json(str: &String) -> Result<Vec<Ball>, Box<dyn std::error::Error>> {
    let file_contents = std::fs::read_to_string(str)?;
    let simulation: Vec<Ball> = serde_json::from_str(&file_contents)?;
    Ok(simulation)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (mut ctx, mut event_loop) = ContextBuilder::new("hello_ggez", "thePalindrome")
        .build()
        .unwrap();
    
    let mut state = Game::new(&mut ctx);

    if args.len() > 1 {
        match load_from_json(args.get(1).unwrap()) {
            Ok(simulation) => {state.circle_vec = simulation;},
            Err(e) =>   {eprintln!("Error occured while loading sim: {}", e);}
        }
    }

    event::run(ctx, event_loop, state);
}
