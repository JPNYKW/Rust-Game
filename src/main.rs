extern crate find_folder;
extern crate freetype as ft;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::f64;

use glutin_window::GlutinWindow as Window;
use graphics::{Context, Graphics, ImageSize};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

fn glyphs(
    face: &mut ft::Face, text: &str
) -> Vec<(Texture, [f64; 2])> {
    let mut x = 10;
    let mut y = 0;
    let mut res = vec![];
    for ch in text.chars() {
        face.load_char(ch as usize, ft::face::LoadFlag::RENDER)
            .unwrap();
        let g = face.glyph();

        let bitmap = g.bitmap();
        let texture = Texture::from_memory_alpha(
            bitmap.buffer(),
            bitmap.width() as u32,
            bitmap.rows() as u32,
            &TextureSettings::new(),
        )
        .unwrap();
        res.push((
            texture,
            [(x + g.bitmap_left()) as f64, (y - g.bitmap_top()) as f64],
        ));

        x += (g.advance().x >> 6) as i32;
        y += (g.advance().y >> 6) as i32;
    }
    res
}

fn render_text<G, T>(
    glyphs: &[(T, [f64; 2])], c: &Context, gl: &mut G
) where
    G: Graphics<Texture = T>,
    T: ImageSize,
{
    for &(ref texture, [x, y]) in glyphs {
        use graphics::*;

        Image::new_color(color::BLACK).draw(texture, &c.draw_state, c.transform.trans(x, y), gl);
    }
}

const GRID_SIZE: f64 = 50.0;
const WINDOW_WIDTH: f64 = 640.0;
const WINDOW_HEIGHT: f64 = 640.0;
const RESULT_TEXT_Y: f64 = (WINDOW_HEIGHT - GRID_SIZE * 8.0) / 4.0;
const STONES_TEXT_Y: f64 = WINDOW_HEIGHT - (WINDOW_HEIGHT - GRID_SIZE * 8.0) / 4.0;

pub struct App {
    gl: GlGraphics,
}

impl App {
    fn render(
        &mut self,
        args: &RenderArgs,
        _board: [[usize; 8]; 8],
        _positions_can_put: [[usize; 8]; 8],
        _is_game_end: bool,
    ) {
        use graphics::*;

        const WHITE_SUB: [f32; 4] = [1.0, 1.0, 1.0, 0.1];
        const WHITE: [f32; 4] = [0.85, 0.85, 0.85, 1.0];
        const BLACK: [f32; 4] = [0.14, 0.14, 0.14, 1.0];
        const GREEN: [f32; 4] = [0.03, 0.51, 0.23, 1.0];

        let square = rectangle::square(-GRID_SIZE * 4.0, -GRID_SIZE * 4.0, GRID_SIZE * 8.0);

        let stone = rectangle::square(0.0, 0.0, 20.0);
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |_c, gl| {
            clear(WHITE, gl);

            let mut _x = 0.0;
            let mut _y = GRID_SIZE * -4.0 + GRID_SIZE;
            let transform = _c.transform.trans(x, y);

            let dx = [
                -GRID_SIZE, GRID_SIZE, GRID_SIZE, GRID_SIZE, GRID_SIZE, -GRID_SIZE, -GRID_SIZE,
                -GRID_SIZE,
            ];
            let dy = [
                GRID_SIZE, GRID_SIZE, GRID_SIZE, -GRID_SIZE, -GRID_SIZE, -GRID_SIZE, -GRID_SIZE,
                GRID_SIZE,
            ];

            rectangle(GREEN, square, transform, gl);

            for _i in 0..7 {
                _x = GRID_SIZE * -4.0 + GRID_SIZE;
                for _j in 0..7 {
                    for _k in 0..4 {
                        line(
                            BLACK,
                            2.0,
                            [
                                _x + dx[_k * 2],
                                _y + dy[_k * 2],
                                _x + dx[_k * 2 + 1],
                                _y + dy[_k * 2 + 1],
                            ],
                            transform,
                            gl,
                        );
                    }
                    _x += GRID_SIZE;
                }
                _y += GRID_SIZE;
            }

            _y = GRID_SIZE * -4.0 + GRID_SIZE;
            for _i in 0..8 {
                _x = GRID_SIZE * -4.0 + GRID_SIZE;
                for _j in 0..8 {
                    if _board[_i][_j] > 0 || _positions_can_put[_i][_j] == 1 {
                        let trans = _c
                            .transform
                            .trans(_x + GRID_SIZE * 6.0 - 15.0, _y + GRID_SIZE * 6.0 - 15.0);

                        circle_arc(
                            if _positions_can_put[_i][_j] == 1 {
                                WHITE_SUB
                            } else if _board[_i][_j] == 1 {
                                WHITE
                            } else {
                                BLACK
                            },
                            10.0,
                            0.0,
                            f64::consts::PI * 1.9999,
                            stone,
                            trans,
                            gl,
                        );
                    }
                    _x += GRID_SIZE;
                }
                _y += GRID_SIZE;
            }

            let assets = find_folder::Search::ParentsThenKids(3, 3)
                .for_folder("assets")
                .unwrap();
            let freetype = ft::Library::init().unwrap();
            let font = assets.join("Geomanist-Regular.otf");
            let mut face = freetype.new_face(&font, 0).unwrap();
            face.set_pixel_sizes(0, 30).unwrap();

            let stones_result = count_stones(_board);
            let white = stones_result[0];
            let black = stones_result[1];

            {
                let glyphs = glyphs(
                    &mut face,
                    &format!(
                        "WHITE: {}{}, BLACK: {}{} ",
                        if white < 10 { "0" } else { "" },
                        white,
                        if black < 10 { "0" } else { "" },
                        black
                    ),
                );

                render_text(
                    &glyphs,
                    &_c.trans(WINDOW_WIDTH / 2.0 - 150.0, STONES_TEXT_Y + 15.0),
                    gl,
                );
            }

            if _is_game_end {
                let judge = get_judgement(stones_result);
                let glyphs = glyphs(&mut face, &format!("{}! ", judge));

                render_text(
                    &glyphs,
                    &_c.trans(
                        WINDOW_WIDTH / 2.0 - if judge == "DRAW" { 55.0 } else { 100.0 },
                        RESULT_TEXT_Y + 15.0,
                    ),
                    gl,
                );
            }
        });
    }
}

fn count_stones(
    _board: [[usize; 8]; 8]
) -> [usize; 2] {
    let mut stones: [usize; 2] = [0; 2];

    for i in 0..8 {
        for j in 0..8 {
            if _board[i][j] > 0 {
                stones[_board[i][j] - 1] += 1;
            }
        }
    }

    stones
}

fn get_reversed_board(
    _x: usize,
    _y: usize,
    _stone: usize,
    _board: [[usize; 8]; 8],
) -> [[usize; 8]; 8] {
    let opponent_stone = if _stone == 1 { 2 } else { 1 };
    let dx: [i32; 8] = [0, -1, -1, -1, 0, 1, 1, 1];
    let dy: [i32; 8] = [1, 1, 0, -1, -1, -1, 0, 1];
    let mut new_board = _board;

    for id in 0..8 {
        let mut _x_pos = _x as i32 + dx[id];
        let mut _y_pos = _y as i32 + dy[id];
        if _x_pos < 0 || _x_pos > 7 || _y_pos < 0 || _y_pos > 7 {
            continue;
        }

        if _board[_y_pos as usize][_x_pos as usize] == opponent_stone {
            let mut flag = true;
            let mut count_max = 0;

            loop {
                count_max += 1;
                _x_pos += dx[id];
                _y_pos += dy[id];

                if _x_pos < 0
                    || _x_pos > 7
                    || _y_pos < 0
                    || _y_pos > 7
                    || _board[_y_pos as usize][_x_pos as usize] == 0
                {
                    flag = false;
                    break;
                } else if _board[_y_pos as usize][_x_pos as usize] == _stone {
                    break;
                }
            }

            if flag {
                _x_pos = _x as i32;
                _y_pos = _y as i32;

                for _i in 0..count_max {
                    _x_pos += dx[id];
                    _y_pos += dy[id];
                    new_board[_y_pos as usize][_x_pos as usize] = _stone;
                }
            }
        }
    }

    new_board
}

fn get_positions_can_put(
    _stone: usize,
    _board: [[usize; 8]; 8]
) -> [[usize; 8]; 8] {
    let mut positions: [[usize; 8]; 8] = [[0; 8]; 8];
    for y in 0..8 {
        for x in 0..8 {
            if _board[y][x] == 0 {
                positions[y][x] = if _board == get_reversed_board(x, y, _stone, _board) {
                    0
                } else {
                    1
                };
            }
        }
    }

    positions
}

fn get_judgement(
    _stones: [usize; 2]
) -> String {
    let white = _stones[0];
    let black = _stones[1];

    (if white == black {
        "DRAW"
    } else {
        if white > black {
            "WHITE WON!"
        } else {
            "BLACK WON"
        }
    })
    .to_string()
}

fn main() {
    let mut skip_count = 0;
    let mut id_x: f64 = 0.0;
    let mut id_y: f64 = 0.0;
    let mut is_game_end = false;
    let mut is_black_turn = true;
    let mut board: [[usize; 8]; 8] = [[0; 8]; 8];
    board[3][3] = 1;
    board[3][4] = 2;
    board[4][3] = 2;
    board[4][4] = 1;

    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Reversi v1.2", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App {
        gl: GlGraphics::new(opengl),
    };

    let mut events = Events::new(EventSettings::new());
    let mut cursor = [0.0, 0.0];

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            let positions_can_put = get_positions_can_put(if is_black_turn { 2 } else { 1 }, board);
            if positions_can_put == [[0; 8]; 8] {
                is_black_turn = !is_black_turn;
                if skip_count == 1 {
                    skip_count = 2;
                    is_game_end = true;
                    println!("\nFinished the game!");

                    let result = count_stones(board);
                    println!("{} - {}, {}!", result[0], result[1], get_judgement(result));
                } else if skip_count == 0 {
                    skip_count = 1;
                    println!("Skip {}!", if !is_black_turn { "BLACK" } else { "WHITE" });
                }
            } else {
                skip_count = 0;
            }

            app.render(&r, board, positions_can_put, is_game_end);
        }

        if let Some(Button::Mouse(button)) = e.press_args() {
            if button == piston::MouseButton::Left {
                if id_x < 0.0 || id_x > 8.0 || id_y < 0.0 || id_y > 8.0 {
                    continue;
                }
                let u_id_x: usize = id_x as usize;
                let u_id_y: usize = id_y as usize;

                if board[u_id_y][u_id_x] > 0 {
                    println!("You can't put there.");
                } else {
                    let stone: usize = if is_black_turn { 2 } else { 1 };

                    if board == get_reversed_board(u_id_x, u_id_y, stone, board) {
                        println!("You can't put there.");
                    } else {
                        board[u_id_y][u_id_x] = stone;
                        board = get_reversed_board(u_id_x, u_id_y, stone, board);
                        is_black_turn = !is_black_turn;
                        println!("{:?}", count_stones(board));
                    }
                }
            }
        }

        e.mouse_cursor(|pos| {
            cursor = pos;
            let left_x = WINDOW_WIDTH / 2.0 - GRID_SIZE * 4.0;
            let top_y = WINDOW_HEIGHT / 2.0 - GRID_SIZE * 4.0;
            let (normal_x, normal_y) = (pos[0] - left_x, pos[1] - top_y);
            id_x = normal_x / GRID_SIZE;
            id_y = normal_y / GRID_SIZE;
        });
    }
}
