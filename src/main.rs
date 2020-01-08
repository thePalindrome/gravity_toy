use ggez::*;
use ggez::input::mouse::MouseButton;
use ggez::input::keyboard::KeyCode;

#[derive(Copy,Clone)]
struct Ball {
    x: f32,
    y: f32,
    xv: f32,
    yv: f32,
    stationary: bool,
}

impl Ball {
    fn new(x: f32, y: f32) -> Ball {
        Ball{ x: x, y: y, xv: 0.0, yv: 0.0, stationary: false}
    }
    fn distance_from(&self, other: Ball) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

struct Game {
    circle_vec: Vec<Ball>,
    stationary: bool,
    xv: f32,
    yv: f32,
}

impl Game {
    fn new(_ctx: &mut Context) -> Game {
        Game{ circle_vec: Vec::new(), stationary: false, xv: 0.0, yv: 0.0 }
    }
}

impl ggez::event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, 60) {
            for i in 0..self.circle_vec.len() {
                if self.circle_vec[i].stationary { continue; }
                for j in 0..self.circle_vec.len() {
                    if i == j {continue;}
                    
                    // TODO: Set some gravity
                    // θ = tan-1 ( y / x )
                    // x = r × cos( θ )
                    // y = r × sin( θ )

                    let distance_mod = (1.0 / (self.circle_vec[i].distance_from(self.circle_vec[j]))).min(2.0).max(0.0);
                    let angle = (self.circle_vec[j].y - self.circle_vec[i].y).atan2(self.circle_vec[j].x - self.circle_vec[i].x);

                    self.circle_vec[i].xv += distance_mod * angle.cos();
                    self.circle_vec[i].yv += distance_mod * angle.sin();
                }
            }
            for circle in self.circle_vec.iter_mut() {
                circle.x += circle.xv;
                circle.y += circle.yv;
            }
        }
        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        if self.xv != 0.0 {
            graphics::queue_text(ctx, &graphics::Text::new(format!("xv: {}", self.xv)),mint::Point2{x:0.0,y:0.0},Some(graphics::DrawParam::default().color));
        }
        if self.yv != 0.0 {
            graphics::queue_text(ctx, &graphics::Text::new(format!("yv: {}", self.yv)),mint::Point2{x:0.0,y:16.0},Some(graphics::DrawParam::default().color));
        }
        if self.stationary {
            graphics::queue_text(ctx, &graphics::Text::new("Placing Static Node"),mint::Point2{x:0.0,y:32.0},Some(graphics::DrawParam::default().color));
        }
        graphics::draw_queued_text(ctx, graphics::DrawParam::default(), None, graphics::FilterMode::Linear)?;
        for circle in self.circle_vec.iter() {
            let circle_blit = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                mint::Point2{x:circle.x, y:circle.y},
                10.0,
                0.1,
                graphics::Color::new(((circle.xv.abs().hypot(circle.yv.abs())) / 10.0).min(1.0), 0.2, 0.2, 1.0),
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
            _ => ()
        }
    }
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
