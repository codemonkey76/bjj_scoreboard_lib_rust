use std::thread;
use std::time::{Duration, SystemTime};

fn main() {
    let shane = Competitor::new("Shane", "Poppleton", "Fight Club Jiu-Jitsu", Country::Australia);
    let ronaldo = Competitor::new("Ronaldo", "Mendes Dos Santos", "Caza BJJ", Country::Brazil);

    let bjj_match = BJJMatch::new(shane, ronaldo, 10, 1, 1);


    bjj_match.start();
    println!("Sleeping for 5 seconds...");
    thread::sleep(Duration::from_secs(5));
    println!("Done sleeping!");
    bjj_match.get_time_remaining();
    println!("Sleeping for 5 seconds...");
    thread::sleep(Duration::from_secs(5));
    println!("Done sleeping!");
    

    println!("Hello World");

}

#[derive(Default)]
struct MatchScore {
    competitor_one_score: PlayerScore,
    competitor_two_score: PlayerScore,
    time_remaining_milliseconds: usize,
}

#[derive(Default)]
struct PlayerScore {
    points: usize,
    advantages: usize,
    penalties: usize
}


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
            time: MatchTime::default()
        }
    }
}

struct MatchInformation {
    competitor_one: Competitor,
    competitor_two: Competitor,
    match_time_minutes: usize,
    mat_number: usize,
    fight_number: usize
}

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

enum Country {
    Australia,
    Brazil,
    UnitedStates
}

#[derive(Default)]
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
                    true => self.time_elapsed_millis + SystemTime::now().duration_since(start_time.clone()).unwrap_or(Duration::new(0, 0)).as_millis() as usize,
                    false => self.time_elapsed_millis
                }
            },
            None => self.time_elapsed_millis
        };

        self.duration_millis.saturating_sub(elapsed)
    }
    pub fn start(&mut self) {
        if self.running {
            return;
        }

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