use egui::{self};
use std::{string, time::{Duration, Instant}};
use interprocess::local_socket::{prelude::*, GenericFilePath, GenericNamespaced, Stream};
use std::io::{prelude::*, BufReader};
use serde;
use cheess::{GameMode, BitBoard};

const FIGURES: [&str; 13] = [
    "♚", "♛", "♜", "♝", "♞", "♟", "", "♙", "♘", "♗", "♖", "♕", "♔",
];


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ChessApp {
    #[serde(skip_serializing, skip_deserializing)]
    label: String,
    #[serde(skip_serializing, skip_deserializing)]
    mode_selected: Option<GameMode>,
    #[serde(skip_serializing, skip_deserializing)]
    white_timer: Duration,
    #[serde(skip_serializing, skip_deserializing)]
    black_timer: Duration,
    #[serde(skip_serializing, skip_deserializing)]
    timer: Option<Instant>,
}

impl Default for ChessApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "(#,#) (#,#)".to_owned(),
            mode_selected: None,
            white_timer: Duration::from_secs(300),
            black_timer: Duration::from_secs(300),
            timer: None,
        }
    }
}

impl ChessApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    // pub fn connect() -> std::io::Result<()> {
        // let name = if GenericNamespaced::is_supported() {
        //     "example.sock".to_ns_name::<GenericNamespaced>()?
        // } else {
        //     "/tmp/example.sock".to_fs_name::<GenericFilePath>()?
        // };
        // let mut buffer = String::with_capacity(128);
        // // blocks until connection or fail
        // let conn = Stream::connect(name)?;
        // let mut conn = BufReader::new(conn);
        // // blocks until message sent or error
        // conn.get_mut().write_all(b"Hello from client!\n")?;
        // conn.read_line(&mut buffer)?;
        
        // println!{"Server responded: {buffer}"}

        // Ok(())
    // }

    pub fn update_timer(&mut self) {
        let time_difference = self.timer.expect("timer was none when game opened").elapsed();
        self.timer = Some(Instant::now());
        self.white_timer = self.white_timer.saturating_sub(time_difference);
        self.black_timer = self.black_timer.saturating_sub(time_difference);
        //recieve clock black and white from seperate thread to update visible clock
    }
}

impl eframe::App for ChessApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
                
        egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            
            
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Exit").clicked() {
                            if let Err(e) = exit(ctx) {
                                eprintln!("Error exiting program: {e}")
                            }
                            // ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        } 
                        if ui.button("Resign").clicked(){
                            if let Err(e) = resign() {
                                eprintln!("Error resigning: {e}")
                            }
                            //declare winner
                        }
                        if ui.button("New Game").clicked(){
                            if let Err(e) = new_game() {
                                eprintln!("Error new gaming: {e}")
                            }
                            self.mode_selected = None
                            //redo viewport to default
                        }

                    });
                    ui.add_space(16.0);
                }

                // egui::widgets::global_theme_preference_buttons(ui); /* system theme button */
                egui::widgets::global_theme_preference_switch(ui);
            });
        });

        if self.mode_selected.is_none() {        
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.heading("Select Game Mode");
                if ui.button("Default").clicked() {
                    self.mode_selected = Some(GameMode::Default);
                    self.timer = Some(Instant::now());
                    //change backend to default game mode
                    if let Err(e) = send_mode(self.mode_selected.unwrap()) {
                        eprintln!("Error sending mode: {e}");
                    }
                };
                if ui.button("Blitz").clicked() {
                    self.mode_selected = Some(GameMode::Blitz);
                    self.timer = Some(Instant::now());
                    //change backened to blitz game mode
                    if let Err(e) = send_mode(self.mode_selected.unwrap()) {
                        eprintln!("Error sending mode: {e}");
                    }
                };
            });

        } else {



        
        // egui::CentralPanel::default().show(&ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            egui::TopBottomPanel::top("board").min_height(400.0).show(&ctx, |ui| {
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    draw_board(ui);
                });
            });
            egui::CentralPanel::default().show(&ctx ,|ui| {
            ui.heading("Where my moves at?");
            //Turn clock goes here
            ui.horizontal(|ui| {
                self.update_timer();
                ui.label("Move Clock");
                ui.label(
                    format!(
                        "White: {:.0?} : Black: {:.0?}",
                        self.white_timer,
                        self.black_timer,
                    )
                );
            });
            ui.horizontal(|ui| {
                ui.label("Input '(Origin: x, y) (Destination: x, y)' : ");
                ui.text_edit_singleline(&mut self.label);
                if ui.button("enter").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Err(e) = send_move(&self.label, ctx) {
                        eprintln!("Error sending move: {e}");
                    }
                }
            });
            

            
            ui.separator();
            ui.add_space(16.0);
            //BOARD GOES HERE
    

            // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     powered_by_egui_and_eframe(ui);
            // egui::warn_if_debug_build(ui);
            // });
            ui.ctx().request_repaint_after_secs(1.0);
        
        });
    };
    }
}


fn backend_post(data: &[u8]) -> std::io::Result<String> {
    //send move to back end and update the visuals
    let name = if GenericNamespaced::is_supported() {
        "cheess.sock".to_ns_name::<GenericNamespaced>()?
    } else {
        "/tmp/cheess.sock".to_fs_name::<GenericFilePath>()?
    };
    let mut buffer = String::with_capacity(128);
    // blocks until connection or fail
    let conn = Stream::connect(name)?;
    let mut conn = BufReader::new(conn);
    conn.get_mut().write_all(data)?;
    conn.read_line(&mut buffer)?;
    println!("{buffer}");
    // blocks until message sent or error
    Ok(buffer)
}

    
fn send_mode(mode: GameMode) -> std::io::Result<()> {
    let mut json = serde_json::to_string(&mode)?;
    json.push('\n');
    let _res = backend_post(&json.as_bytes())?;
    Ok(())
}
// we wanna send game mode, and moves nothing else needs to be sent
fn send_move(input: &str, ctx: &egui::Context) -> std::io::Result<()> {
    let mut line = input.to_string().into_bytes();
    line.push(b'\n');
    let _res = backend_post(&line)?;

    if input == "exit" {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    Ok(())
}

fn resign() ->std::io::Result<()> {
    //run resign on back end and update visuals
    let mut line = "resign".to_string().into_bytes();
    line.push(b'\n');
    let _res = backend_post(&line)?;
    Ok(())
}


fn draw_board(ui: &mut egui::Ui) {
    let available_size = ui.available_size();
    let central_panel_rect = ui.min_rect();
    let center_x = central_panel_rect.center().x;
    let center_y = central_panel_rect.center().y;
    let board_size = available_size.min_elem();
    let square_size = board_size / 8.0;
    let board_top_left = egui::Pos2 {
        x: center_x - (4.0 * square_size),
        y: center_y - (4.0 * square_size),
    };
    let mut responses = Vec::new();


    for row in 0..8 {
        for col in 0..8 {
            let color = tile_colour(row, col);
            // let color = egui::Color32::from_rgb(0, 0, 0);
            let top_left = egui::Pos2 {
                x: board_top_left.x + (col as f32 * square_size),
                y: board_top_left.y + (row as f32 * square_size),
            };
            let bottom_right = egui::Pos2 {
                x: top_left.x + square_size,
                y: top_left.y + square_size,
            };
            let rect = egui::Rect::from_two_pos(top_left, bottom_right);
            // let response = ui.allocate_rect(rect, egui::Sense::click());

            responses.push((rect, color, col, row));
        }
    }
    
    let painter = ui.painter();
    for (rect, color, col, row) in responses {
        painter.rect_filled(rect, 0.0, color);
        let text_pos = rect.center();
        let piece = FIGURES[5 as usize];
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    piece,
                    egui::FontId::proportional(square_size * 0.9),
                    egui::Color32::BLACK,
                );

    }
    
    
    
}

fn tile_colour(x: usize, y: usize) -> egui::Color32 {
    if (x + y) % 2 != 0 {
        egui::Color32::from_rgb(173,189,143)
    } else {
        egui::Color32::from_rgb(111,143,114)
    }
}


fn new_game() -> std::io::Result<()> {
    //run newgame creation on back end and update visuals
    let mut line = "reset".to_string().into_bytes();
    line.push(b'\n');
    let _res = backend_post(&line)?;
    Ok(())
}

fn exit(ctx: &egui::Context) -> std::io::Result<()> {
    let mut line = "exit".to_string().into_bytes();
    line.push(b'\n');
    let _res = backend_post(&line)?;
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    Ok(())
}