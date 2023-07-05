use std::default;
use std::io::{stdout, Write};
use std::time::Duration;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use anyhow::Result;
use crossterm::{event, ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode};
use crossterm::style::Print;
use bjj_scoreboard::{BJJMatch, Competitor, CompetitorNumber, Country, MatchInformation, MatchState};
use eframe::egui;
use crate::AppState::{NewMatchDialog, Normal};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        ..Default::default()
    };
    eframe::run_native(
        "BJJ Scoreboard",
        options,
        Box::new(|_cc| Box::<BjjScoreboard>::default())
    )
}

enum AppState {
    NewMatchDialog,
    Normal
}

struct BjjScoreboard {
    bjj_match: BJJMatch,
    app_state: AppState,
    match_dialog_open: bool
}

impl Default for BjjScoreboard {
    fn default() -> Self {
        Self {
            bjj_match: Default::default(),
            app_state: NewMatchDialog,
            match_dialog_open: true
        }
    }
}

impl eframe::App for BjjScoreboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.app_state {
            AppState::NewMatchDialog => {
                self.draw_new_match_modal(ctx)
            },
            AppState::Normal => {
                self.draw_match_screen(ctx);
            }
        }

    }
}

impl BjjScoreboard {
    fn draw_match_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BJJ Scoreboard");
            ui.heading(format_millis(self.bjj_match.time.get_remaining_time_milliseconds()))
            // ui.horizontal(|ui| {
                // let name_label = ui.label("Competitor 1: ");
                // ui.text_edit_singleline(&mut self.bjj_match.competitor_one_name).labelled_by(name_label.id);
            // });
        });
    }

    fn draw_competitor_dialog(heading: &str, competitor: &mut Competitor, ui: &mut egui::Ui) {
        ui.heading(heading);
        ui.end_row();

        let first = ui.label("First Name");
        ui.text_edit_singleline(&mut competitor.first_name).labelled_by(first.id);
        ui.end_row();

        let last = ui.label("Last Name");
        ui.text_edit_singleline(&mut competitor.last_name).labelled_by(last.id);
        ui.end_row();

        let team = ui.label("Team");
        ui.text_edit_singleline(&mut competitor.team_name).labelled_by(team.id);
        ui.end_row();

        let country = ui.label("Country");
        egui::ComboBox::from_id_source(country.id)
            .selected_text(format!("{:?}", competitor.country))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                ui.selectable_value(&mut competitor.country, Country::Australia, "Australia");
                ui.selectable_value(&mut competitor.country, Country::Brazil, "Brazil");
            });
        ui.end_row();
    }

    fn draw_match_info_dialog(heading: &str, info: &mut MatchInformation, ui: &mut egui::Ui) {
        ui.heading(heading);
        ui.end_row();

        let match_time = ui.label("Match Duration (mins)");
        ui.add(egui::DragValue::new(&mut info.match_time_minutes).speed(0.1).clamp_range(1..=30)).labelled_by(match_time.id);
        ui.end_row();

        let mat_num = ui.label("Mat Number");
        ui.add(egui::DragValue::new(&mut info.mat_number).speed(0.1).clamp_range(1..=20)).labelled_by(mat_num.id);
        ui.end_row();

        let fight_num = ui.label("Fight Number");
        ui.add(egui::DragValue::new(&mut info.fight_number).speed(0.1).clamp_range(1..=30)).labelled_by(fight_num.id);
        ui.end_row();
    }

    fn draw_new_match_modal(&mut self, ctx: &egui::Context) {
        egui::Window::new("Match Settings")
            .open(&mut self.match_dialog_open)
            .show(ctx,|ui| {
                    egui::Grid::new("my_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            BjjScoreboard::draw_competitor_dialog("Competitor One", &mut self.bjj_match.info.competitor_one, ui);
                            ui.separator();
                            ui.end_row();
                            BjjScoreboard::draw_competitor_dialog("Competitor Two", &mut self.bjj_match.info.competitor_two, ui);
                            ui.separator();
                            ui.end_row();
                            BjjScoreboard::draw_match_info_dialog("Match Information", &mut self.bjj_match.info, ui);
                            ui.separator();
                            ui.end_row();
                            if ui.add(egui::Button::new("Start Match")).clicked() {
                                self.app_state = AppState::Normal;
                                self.bjj_match.start();
                            }
                        });
                }
            );
    }


}

fn app() -> Result<()> {
    let shane = Competitor::new("Shane", "Poppleton", "Fight Club Jiu-Jitsu", Country::Australia);
    let ronaldo = Competitor::new("Ronaldo", "Mendes Dos Santos", "Caza BJJ", Country::Brazil);

    let mut bjj_match = BJJMatch::new(shane, ronaldo, 10, 1, 1);


    bjj_match.start();
    println!("Match started, entering raw mode");
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    loop {

        let match_state = bjj_match.get_match_state();

        if match_state == MatchState::Finished {
            break;
        }


        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => {
                        bjj_match.add_points(2, CompetitorNumber::One);
                    }
                    KeyCode::Char('w') => {
                        bjj_match.add_points(3, CompetitorNumber::One);
                    }
                    KeyCode::Char('e') => {
                        bjj_match.add_points(4, CompetitorNumber::One);
                    }
                    KeyCode::Char('r') => {
                        bjj_match.add_advantage(CompetitorNumber::One);
                    }
                    KeyCode::Char('t') => {
                        bjj_match.add_penalty(CompetitorNumber::One);
                    }
                    KeyCode::Char('y') => {
                        bjj_match.subtract_point(CompetitorNumber::One);
                    }
                    KeyCode::Char('u') => {
                        bjj_match.subtract_advantage(CompetitorNumber::One);
                    }
                    KeyCode::Char('i') => {
                        bjj_match.subtract_penalty(CompetitorNumber::One);
                    }
                    KeyCode::Char('a') => {
                        bjj_match.add_points(2, CompetitorNumber::Two);
                    }
                    KeyCode::Char('s') => {
                        bjj_match.add_points(3, CompetitorNumber::Two);
                    }
                    KeyCode::Char('d') => {
                        bjj_match.add_points(4, CompetitorNumber::Two);
                    }
                    KeyCode::Char('f') => {
                        bjj_match.add_advantage(CompetitorNumber::Two);
                    }
                    KeyCode::Char('g') => {
                        bjj_match.add_penalty(CompetitorNumber::Two);
                    }
                    KeyCode::Char('h') => {
                        bjj_match.subtract_point(CompetitorNumber::Two);
                    }
                    KeyCode::Char('j') => {
                        bjj_match.subtract_advantage(CompetitorNumber::Two);
                    }
                    KeyCode::Char('k') => {
                        bjj_match.subtract_penalty(CompetitorNumber::Two);
                    }
                    KeyCode::Char(' ') => {
                        bjj_match.toggle_start_stop();
                    }
                    KeyCode::Esc => {
                        println!("Escape key is pressed. Exiting...");
                        break;
                    }
                    _ => {}
                }
            }
        } else {
            // No event in the past 100ms
        }



        draw_scoreboard(&bjj_match)?;
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    println!("Raw Mode disabled");


    Ok(())
}

fn println_at(row: u16, text: String) -> Result<()> {
    stdout().queue(crossterm::cursor::MoveTo(0, row))?;
    stdout().queue(Print(text))?;
    Ok(())
}

fn draw_scoreboard(bjj_match: &BJJMatch) -> Result<()> {
    stdout().execute(Clear(ClearType::All))?;
    let comp1 = &bjj_match.info.competitor_one;
    let comp2 = &bjj_match.info.competitor_two;
    let score1 = &bjj_match.score.competitor_one_score;
    let score2 = &bjj_match.score.competitor_two_score;

    println_at(0, format!("{} {}", comp1.first_name, comp1.last_name))?;
    println_at(1, format!("Points: {}    Advantages: {}    Penalties: {}", score1.points, score1.advantages, score1.penalties))?;
    println_at(3, format!("{} {}", comp2.first_name, comp2.last_name))?;
    println_at(4, format!("Points: {}    Advantages: {}    Penalties: {}", score2.points, score2.advantages, score2.penalties))?;
    println_at(6, format!("Time remaining: {}", format_millis(bjj_match.time.get_remaining_time_milliseconds())))?;

    stdout().flush()?;
    Ok(())
}

pub fn format_millis(millis: usize) -> String {
    let hours = millis / 3_600_000;
    let minutes = (millis % 3_600_000) / 60_000;
    let seconds = (millis % 60_000) / 1_000;
    let milliseconds = millis % 1_000;

    format!("{:01}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds)
}