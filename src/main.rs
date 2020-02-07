use ggez::*;
use ggez::input::mouse::MouseButton;
use ggez::input::keyboard::KeyCode;
use std::collections::VecDeque;

#[derive(Clone)]
struct Ball {
    x: f32,
    y: f32,
    xv: f32,
    yv: f32,
    stationary: bool,
    mass: f32,
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
}

impl Game {
    fn new(_ctx: &mut Context) -> Game {
        Game{ circle_vec: Vec::new(), stationary: false, xv: 0.0, yv: 0.0, mass: 10.0, show_help: false, draw_trails: false, running: true }
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
        graphics::clear(ctx, graphics::BLACK);

        draw_text(ctx, "Press F1 for help".to_string(), 128.0, 0.0);
        if self.show_help {
            draw_text(ctx,"Left click to place node\nLeft Arrow/Right Arrow to modify X velocity\nUp Arrow/Down Arrow to modify Y velocity\nLeft Shift/Right Shift to modify mass of node\nEnter/Return to reset velocity and mass to default values\nTab to pause simulation\nSpace to toggle \"Static\" mode\nBackspace to clear all nodes\nF2 to toggle trails (performance heavy)\nEscape to exit".to_string(), 128.0, 16.0);
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
        for circle in self.circle_vec.iter() {
            let capacity = circle.trail.capacity() as f32;
            let mass = circle.mass.log10();
            let color = graphics::Color::new(((circle.xv.abs().hypot(circle.yv.abs())) / 10.0).min(1.0), 0.2, 0.2, 1.0);
            if self.draw_trails {
                for (id, trail) in circle.trail.iter().enumerate() {
                    let trail_modifier = 0.9 - ((id + 1) as f32 / capacity) as f32;
                    if trail_modifier < 0.0 { continue; }
                    let trail_blit = graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        mint::Point2{x:trail.0, y:trail.1},
                        10.0 * mass * trail_modifier,
                        0.1,
                        color,
                    )?;
                    graphics::draw(ctx, &trail_blit, graphics::DrawParam::default())?;
                }
            }
            let circle_blit = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                mint::Point2{x:circle.x, y:circle.y},
                10.0 * mass,
                0.1,
                color,
            )?;
            graphics::draw(ctx, &circle_blit, graphics::DrawParam::default())?;
        }
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == ggez::input::mouse::MouseButton::Left {
            let mut ball = Ball::new(x,y);
            ball.stationary = self.stationary;
            ball.xv = self.xv;
            ball.yv = self.yv;
            ball.mass = self.mass;
            self.circle_vec.push(ball);
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
            _ => ()
        }
    }
}

fn draw_text(ctx: &mut Context, str: String, x: f32, y: f32) {
    graphics::queue_text(ctx, &graphics::Text::new(str),mint::Point2{x:x,y:y},Some(graphics::DrawParam::default().color));
}

fn main() {
    let c = conf::Conf::new();
    let (mut ctx, mut event_loop) = ContextBuilder::new("hello_ggez", "thePalindrome")
        .conf(c)
        .build()
        .unwrap();
    
    let mut state = Game::new(&mut ctx);
    event::run(&mut ctx, &mut event_loop, &mut state).unwrap();
}
