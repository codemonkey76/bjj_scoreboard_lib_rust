use std::io::{stdout, Write};
use std::time::Duration;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use anyhow::Result;
use crossterm::{event, ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode};
use crossterm::style::Print;
use bjj_scoreboard::{BJJMatch, Competitor, CompetitorNumber, Country, MatchState};


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