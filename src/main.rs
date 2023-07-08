
use std::io::{stdout, Write};
use std::time::Duration;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use anyhow::Result;
use crossterm::{event, ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode};
use crossterm::style::Print;
use bjj_scoreboard::{BJJMatch, Competitor, CompetitorNumber, Country, MatchInformation, MatchState};
use eframe::egui;
use eframe::egui::{Align2, Color32, FontFamily, Pos2, Rounding};
use eframe::emath::Rect;
use egui_extras::Size;
use egui_grid::GridBuilder;
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
                self.draw_active_match_screen(ctx);
                ctx.request_repaint();
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

    fn draw_active_match_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }


    fn ui(&mut self, ui: &mut egui::Ui) {
        let match_grid = calc_grids(ui.clip_rect());

        ui.painter().rect_filled(match_grid.full, Rounding::none(), Color32::BLACK);

        ui.painter().rect_filled(match_grid.competitor_one.flag, Rounding::none(), Color32::LIGHT_GREEN);
        ui.painter().rect_filled(match_grid.competitor_one.name, Rounding::none(), Color32::DARK_GREEN);
        ui.painter().rect_filled(match_grid.competitor_one.team, Rounding::none(), Color32::DARK_GRAY);
        ui.painter().rect_filled(match_grid.competitor_one.points, Rounding::none(), Color32::RED);


        ui.painter().rect_filled(match_grid.competitor_two.full, Rounding::none(), Color32::DARK_BLUE);

        ui.painter().rect_filled(match_grid.competitor_two.flag, Rounding::none(), Color32::BROWN);
        ui.painter().rect_filled(match_grid.competitor_two.name, Rounding::none(), Color32::YELLOW);
        ui.painter().rect_filled(match_grid.competitor_two.team, Rounding::none(), Color32::DARK_GRAY);
        ui.painter().rect_filled(match_grid.competitor_two.points, Rounding::none(), Color32::BLUE);

        ui.painter().rect_filled(match_grid.time.time, Rounding::none(), Color32::BLUE);
        ui.painter().rect_filled(match_grid.time.fight_info_heading, Rounding::none(), Color32::LIGHT_GREEN);
        ui.painter().rect_filled(match_grid.time.fight_info_sub_heading, Rounding::none(), Color32::GREEN);
        ui.painter().rect_filled(match_grid.time.logo, Rounding::none(), Color32::BROWN);


        // Specify the text color
        let color = Color32::from_rgb(255, 0, 0);

        let font = egui::FontId {
            size: 32.0,
            ..Default::default()
        };


        ui.painter().text(match_grid.time.time.center(), Align2::CENTER_CENTER, format_millis(self.bjj_match.time.get_remaining_time_milliseconds()), font, color);


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

#[derive(Debug)]
struct MatchGrid {
    full: Rect,
    competitor_one: CompetitorGrid,
    competitor_two: CompetitorGrid,
    time: TimeGrid
}

#[derive(Debug)]
struct TimeGrid {
    full: Rect,
    time: Rect,
    fight_info_heading: Rect,
    fight_info_sub_heading: Rect,
    logo: Rect
}
#[derive(Debug)]
struct CompetitorGrid {
    full: Rect,
    main: Rect,
    left: Rect,
    right: Rect,
    comp: Rect,
    flag: Rect,
    name: Rect,
    team: Rect,
    points: Rect,
    advantages: Rect,
    penalties: Rect
}

fn calc_grids(rect: Rect) -> MatchGrid {
    let (top,bottom) = split_vertical(rect, 0.75);
    let (top, middle) = split_vertical(top, 0.5);

    let competitor_one = calc_competitor_grid(top);
    let competitor_two = calc_competitor_grid(middle);

    let time = calc_time_grid(bottom);

    MatchGrid {
        full: rect,
        competitor_one,
        competitor_two,
        time
    }
}

fn calc_time_grid(rect: Rect) -> TimeGrid {
    let (left, logo) = split_horizontal(rect, 5.0 / 6.0);
    let (time, fight_info) = split_horizontal(left, 1.0 / 3.0);
    let (fight_info_heading, fight_info_sub_heading) = split_vertical(fight_info, 0.5);

    TimeGrid {
        full: rect,
        time,
        fight_info_heading,
        fight_info_sub_heading,
        logo
    }
}

fn calc_competitor_grid(rect: Rect) -> CompetitorGrid {
    let (main, points) = split_horizontal(rect, 5.0 / 6.0);
    let (left, right) = split_horizontal(main, 10.0 / 11.0);
    let (comp, team) = split_vertical(left, 2.0 / 3.0);
    let (flag, name) = split_horizontal(comp, 1.0 / 8.0);
    let (advantages, penalties) = split_vertical(right, 0.5);

    CompetitorGrid {
        full: rect,
        main,
        left,right,
        comp,
        flag,
        name,
        team,
        points,
        advantages,
        penalties
    }
}

fn split_horizontal(rect: Rect, at: f32) -> (Rect, Rect) {
    let new_x = rect.min.x + (rect.max.x - rect.min.x) * at;

    (
        Rect::from_min_max(rect.min, Pos2::new(new_x, rect.max.y)),
        Rect::from_min_max(Pos2::new(new_x, rect.min.y), rect.max)
    )
}

fn split_fixed_horizontal(rect: Rect, at: f32) -> (Rect, Rect) {
    let new_x = rect.min.x + at;

    (
        Rect::from_min_max(rect.min, Pos2::new(new_x, rect.max.y)),
        Rect::from_min_max(Pos2::new(new_x, rect.min.y), rect.max)
    )
}

fn split_vertical(rect: Rect, at: f32) -> (Rect, Rect) {
    let new_y = rect.min.y + (rect.max.y - rect.min.y) * at;

    (
        Rect::from_min_max(rect.min, Pos2::new(rect.max.x, new_y)),
        Rect::from_min_max(Pos2::new(rect.min.x, new_y), rect.max)
    )
}

fn split_fixed_vertical(rect: Rect, at: f32) -> (Rect, Rect) {
    let new_y = rect.min.y + at;

    (
        Rect::from_min_max(rect.min, Pos2::new(rect.max.x, new_y)),
        Rect::from_min_max(Pos2::new(rect.min.x, new_y), rect.max)
    )
}
#[cfg(test)]
mod tests {
    use eframe::egui::{Pos2, Rect};
    use crate::split_vertical;

    #[test]
    fn test_split_vertical() {
        let p1 = Pos2::new(0.0, 180.0);
        let p2 = Pos2::new(484.0, 360.0);
        let rect = Rect::from_min_max(p1, p2);

        let (rect1, _) = split_vertical(rect, 2.0 / 3.0);

        assert_eq!(rect1.min.x, 0.0);
        assert_eq!(rect1.min.y, 180.0);

        assert_eq!(rect1.max.x, 484.0);
        assert_eq!(rect1.max.y, 300.0);
    }
}

