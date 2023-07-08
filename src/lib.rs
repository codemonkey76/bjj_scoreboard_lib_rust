use std::time::{Duration, SystemTime};

#[derive(Debug, PartialEq)]
pub enum CompetitorNumber {
    One,
    Two
}

#[derive(Debug, PartialEq)]
pub enum MatchState {
    NotStarted,
    InProgress,
    Finished
}

#[derive(Debug, PartialEq)]
enum ScoreField {
    Points,
    Advantages,
    Penalties
}

#[derive(Debug, PartialEq)]
pub enum Country {
    Australia,
    Brazil,
    UnitedStates
}

#[derive(Default ,Debug)]
pub struct MatchScore {
    pub competitor_one_score: PlayerScore,
    pub competitor_two_score: PlayerScore,
    pub time_remaining_milliseconds: usize,
}

#[derive(Default, Debug)]
pub struct PlayerScore {
    pub points: usize,
    pub advantages: usize,
    pub penalties: usize
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

#[derive(Default, Debug)]
pub struct BJJMatch {
    pub info: MatchInformation,
    pub score: MatchScore,
    pub time: MatchTime
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
        self.time.duration_millis = self.info.match_time_minutes * 60 * 1000;
        self.time.start();
    }

    pub fn toggle_start_stop(&mut self) {
        self.time.toggle_start_stop();
    }
}

#[derive(Debug)]
pub struct MatchInformation {
    pub competitor_one: Competitor,
    pub competitor_two: Competitor,
    pub match_time_minutes: usize,
    pub mat_number: usize,
    pub fight_number: usize
}

impl Default for MatchInformation {
    fn default() -> Self {
        Self {
            competitor_one: Competitor {
                last_name: "One".to_owned(),
                ..Default::default()
            },
            competitor_two: Competitor {
                last_name: "Two".to_owned(),
                ..Default::default()
            },
            match_time_minutes: 5,
            mat_number: 1,
            fight_number: 1
        }
    }
}

#[derive(Debug)]
pub struct Competitor {
    pub first_name: String,
    pub last_name: String,
    pub team_name: String,
    pub country: Country
}

impl Default for Competitor {
    fn default() -> Self {
        Self {
            first_name: "Competitor".to_owned(),
            last_name: "Name".to_owned(),
            team_name: "BJJ Team".to_owned(),
            country: Country::Australia
        }
    }
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


#[derive(Default, Debug)]
pub struct MatchTime {
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