use std::io::{stdout, Write};
use std::thread;
use std::time::{Duration, SystemTime};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use anyhow::Result;
use crossterm::{event, ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode};
use crossterm::style::Print;

fn main() -> Result<()> {
    app()?;

    Ok(())
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

enum CompetitorNumber {
    One,
    Two
}

#[derive(PartialEq)]
enum MatchState {
    NotStarted,
    InProgress,
    Finished
}

#[derive(Debug)]
enum ScoreField {
    Points,
    Advantages,
    Penalties
}

#[derive(Default ,Debug)]
struct MatchScore {
    competitor_one_score: PlayerScore,
    competitor_two_score: PlayerScore,
    time_remaining_milliseconds: usize,
}



#[derive(Default, Debug)]
struct PlayerScore {
    points: usize,
    advantages: usize,
    penalties: usize
}

impl PlayerScore {
    fn subtract(&mut self, field: ScoreField) {
        match field {
            ScoreField::Points => {
                if self.points > 0 {
                    self.points -= 1;
                }
            }
            ScoreField::Advantages => {
                if self.advantages > 0 {
                    self.advantages -= 1;
                }
            }
            ScoreField::Penalties => {
                if self.penalties > 0 {
                    self.penalties -= 1;
                }
            }
        }
    }
}


#[derive(Debug)]
struct BJJMatch {
    info: MatchInformation,
    score: MatchScore,
    time: MatchTime
}

impl BJJMatch {
    pub fn new(competitor_one: Competitor, competitor_two: Competitor, match_time_minutes: usize, mat_number: usize, fight_number: usize) -> BJJMatch {
        BJJMatch{
            info: MatchInformation {
                competitor_one,
                competitor_two,
                match_time_minutes,
                mat_number,
                fight_number
            },
            score: MatchScore::default(),
            time: MatchTime {
                duration_millis: match_time_minutes * 60 * 1000,
                ..Default::default()
            }
        }
    }

    pub fn get_match_state(&self) -> MatchState {
        match self.time.last_started {
            None => MatchState::NotStarted,
            _ => match self.time.get_remaining_time_milliseconds() {
                0 => MatchState::Finished,
                _ => MatchState::InProgress,
            },
        }
    }

    pub fn add_points(&mut self, points: usize, competitor: CompetitorNumber) {
        match competitor {
            CompetitorNumber::One => self.score.competitor_one_score.points += points,
            CompetitorNumber::Two => self.score.competitor_two_score.points += points
        };
    }

    pub fn add_advantage(&mut self, competitor: CompetitorNumber) {
        match competitor {
            CompetitorNumber::One => self.score.competitor_one_score.advantages += 1,
            CompetitorNumber::Two => self.score.competitor_two_score.advantages += 1
        };
    }

    pub fn add_penalty(&mut self, competitor: CompetitorNumber) {
        match competitor {
            CompetitorNumber::One => self.score.competitor_one_score.penalties += 1,
            CompetitorNumber::Two => self.score.competitor_two_score.penalties += 1
        };
    }

    pub fn subtract_point(&mut self, competitor: CompetitorNumber) {
        match competitor {
            CompetitorNumber::One => self.score.competitor_one_score.subtract(ScoreField::Points),
            CompetitorNumber::Two => self.score.competitor_two_score.subtract(ScoreField::Points)
        };
    }

    pub fn subtract_advantage(&mut self, competitor: CompetitorNumber) {
        match competitor {
            CompetitorNumber::One => self.score.competitor_one_score.subtract(ScoreField::Advantages),
            CompetitorNumber::Two => self.score.competitor_two_score.subtract(ScoreField::Advantages)
        };
    }

    pub fn subtract_penalty(&mut self, competitor: CompetitorNumber) {
        match competitor {
            CompetitorNumber::One => self.score.competitor_one_score.subtract(ScoreField::Penalties),
            CompetitorNumber::Two => self.score.competitor_two_score.subtract(ScoreField::Penalties)
        };
    }

    pub fn start(&mut self) {
        self.time.start();
    }

    pub fn toggle_start_stop(&mut self) {
        self.time.toggle_start_stop();
    }
}

#[derive(Debug)]
struct MatchInformation {
    competitor_one: Competitor,
    competitor_two: Competitor,
    match_time_minutes: usize,
    mat_number: usize,
    fight_number: usize
}

#[derive(Debug)]
struct Competitor {
    first_name: String,
    last_name: String,
    team_name: String,
    country: Country
}

impl Competitor {
    pub fn new(first_name: &str, last_name: &str, team_name: &str, country: Country) -> Competitor {
        Competitor {
            first_name: first_name.to_owned(),
            last_name: last_name.to_owned(),
            team_name: team_name.to_owned(),
            country
        }
    }
}

#[derive(Debug)]
enum Country {
    Australia,
    Brazil,
    UnitedStates
}

#[derive(Default, Debug)]
struct MatchTime {
    duration_millis: usize,
    last_started: Option<SystemTime>,
    time_elapsed_millis: usize,
    running: bool
}

impl MatchTime {
    pub fn get_remaining_time_milliseconds(&self) -> usize {
        let elapsed = match &self.last_started {
            Some(start_time) => {
                match self.running {
                    true => {
                        self.time_elapsed_millis + SystemTime::now().duration_since(*start_time).unwrap_or(Duration::new(0, 0)).as_millis() as usize
                    },
                    false => {
                        self.time_elapsed_millis
                    }
                }
            },
            None => {
                self.time_elapsed_millis
            }
        };

        self.duration_millis.saturating_sub(elapsed)
    }

    pub fn toggle_start_stop(&mut self) {
        if self.running {
            self.stop();
        } else {
            self.start();
        }
    }

    pub fn start(&mut self) {
        if self.running {
            return;
        }

        self.running = true;
        self.last_started = Some(SystemTime::now());
    }

    pub fn stop(&mut self) {
        if !self.running {
           return;
        }

        let elapsed = match &self.last_started {
            Some(start_time) => {
                SystemTime::now().duration_since(start_time.clone()).unwrap_or(Duration::new(0,0)).as_millis() as usize
            },
            None => 0
        };

        self.running = false;
        self.time_elapsed_millis += elapsed;
    }
}