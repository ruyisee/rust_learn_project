use bracket_lib::prelude::*;
use bracket_lib::random;

const SCREEN_WIDTH: i32 = 80;
const PLAYER_ABS_X: i32 = 20; // 角色相对于窗口的x位置固定
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

fn main() -> BError {
    let context = BTermBuilder::simple80x50().with_title("~Flappy~").build()?;
    // 启动游戏主循环
    main_loop(context, State::new())
}

// 游戏状态
enum GameMode {
    // 开始菜单
    Menu,
    // 运行中
    Playing,
    // 结束
    End,
}

// 角色
struct Player {
    x: i32,
    y: i32,
    y_real: f32,   // 实际位置
    velocity: f32, // 垂直（Y）方向的速度
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x: x,
            y: y,
            y_real: y as f32,
            velocity: 0.0,
        }
    }
    fn render(&self, ctx: &mut BTerm) {
        ctx.set(PLAYER_ABS_X, self.y, YELLOW, BLUE, to_cp437('@'));
    }
    // 前移与重力下落
    fn update(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y_real += self.velocity;
        self.y = self.y_real as i32;
        self.x += 1;
    }
    // 向上飞
    fn fly(&mut self) {
        self.velocity = -2.0;
    }
}
// 障碍物
struct Obstacle {
    x: i32,
    gap_y: i32, // 通过洞口中间的y坐标
    size: i32,  // 洞口宽度
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = random::RandomNumberGenerator::new();
        Obstacle {
            x: x,
            gap_y: random.range(10, 40),
            size: i32::max(3, 20 - score),
        }
    }

    fn render(&self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, DARK_GREEN, BLACK, to_cp437('|'));
        }
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, DARK_GREEN, BLACK, to_cp437('|'));
        }
    }
    // 碰撞检查
    fn hit_check(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let x_match = player.x + PLAYER_ABS_X == self.x;
        let y_above_match = player.y < self.gap_y - half_size;
        let y_below_match = player.y > self.gap_y + half_size;
        x_match && (y_above_match || y_below_match)
    }
}
// 游戏主逻辑
struct State {
    player: Player,
    frame_time: f32,
    mode: GameMode,
    obstacle: Obstacle,
    score: i32,
}

impl State {
    fn new() -> Self {
        State {
            mode: GameMode::Menu,
            frame_time: 0.0,
            player: Player::new(0, 25),
            score: 0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
        }
    }
    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "~Flappy~");
        ctx.print_centered(7, "Press (P) start new game");
        ctx.print_centered(8, "Press (Q) exit game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => self.quit(ctx),
                _ => {}
            }
        }
    }
    // 重新开始
    fn restart(&mut self) {
        self.player = Player::new(0, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
    }
    fn quit(&mut self, ctx: &mut BTerm) {
        self.mode = GameMode::End;
        ctx.quit()
    }
    fn play(&mut self, ctx: &mut BTerm) {
        // 清理屏幕并制定指定背景颜色
        ctx.cls_bg(NAVY);
        // ctx.frame_time_ms 记录了从上次tick被调用到现在过去了多长时间
        // 这里是记录了上次update到现在过去了多长时间
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.update();

            // 检查玩家是否死亡
            if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_check(&self.player) {
                self.mode = GameMode::End;
            } else {
                if self.player.x + PLAYER_ABS_X == self.obstacle.x {
                    self.score += 1;
                }
                if self.obstacle.x < self.player.x {
                    self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
                }
            }
        }
        // 按空格向上飞
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.fly();
        }
        // 渲染相关
        self.player.render(ctx);
        self.obstacle.render(ctx, self.player.x);
        ctx.print(0, 1, format!("Score: {}", self.score));
    }
    fn game_over(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "~GAME OVER~");
        ctx.print_centered(6, format!("Your Score: {}", self.score));
        ctx.print_centered(7, "Press (P) start new game");
        ctx.print_centered(8, "Press (Q) exit game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => self.quit(ctx),
                _ => {}
            }
        }
    }
}

// 游戏状态中的tick
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),

            GameMode::Playing => self.play(ctx),

            GameMode::End => self.game_over(ctx),
        }
    }
}
