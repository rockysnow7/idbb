use crate::baseball::{AtBatOutcome, Base, BatterDecision, BattingOutcome, Count, EventsSummary, GameStateSummary, Player, RunnerAdvancement};
use std::collections::HashMap;

pub enum Granularity {
    Pitch,
    HalfInning,
}

impl Granularity {
    fn calculate_tension(state_summary: &GameStateSummary) -> usize {
        let inning = state_summary.half_inning.number as usize;
        let run_diff = (state_summary.home_team_runs as i8 - state_summary.visiting_team_runs as i8).abs() as usize;

        if run_diff == 0 {
            f32::INFINITY as usize
        } else {
            inning / run_diff
        }
    }

    pub fn from_state_summary(state_summary: &GameStateSummary) -> Self {
        let tension = Self::calculate_tension(state_summary);

        if tension > 3 {
            Self::HalfInning
        } else {
            Self::Pitch
        }
    }
}

#[derive(Debug)]
pub struct TextEngine {
    all_players: HashMap<String, Player>,
    home_team_name: String,
    visiting_team_name: String,
}

impl TextEngine {
    pub fn new(
        all_players: HashMap<String, Player>,
        home_team_name: String,
        visiting_team_name: String,
    ) -> Self {
        Self {
            all_players,
            home_team_name,
            visiting_team_name,
        }
    }

    fn describe_strike(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        let new_count = format!("{} and {}", new_game_state_summary.count.balls, new_game_state_summary.count.strikes);
        let sentence = match events_summary.batter_decision {
            BatterDecision::Swing => format!("A swing and a miss, it's {new_count}."),
            BatterDecision::Take => format!("Strike looking, it's {new_count}."),
        };
        sentences.push(sentence);
    }

    fn describe_ball(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        sentences.push("Ball.".to_string());
    }

    fn describe_pitch_with_no_at_bat_outcome(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        match events_summary.batting_outcome {
            BattingOutcome::Strike => self.describe_strike(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                sentences,
            ),
            BattingOutcome::Ball => self.describe_ball(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                sentences,
            ),
            _ => unreachable!(),
        }
    }

    fn describe_strikeout(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        let sentence = match events_summary.batter_decision {
            BatterDecision::Swing => format!(
                "He swings and he misses, {} strikes out {}.",
                prev_game_state_summary.pitcher,
                prev_game_state_summary.batter,
            ),
            BatterDecision::Take => format!(
                "Strike three, {} strikes out {}.",
                prev_game_state_summary.pitcher,
                prev_game_state_summary.batter,
            ),
        };
        sentences.push(sentence);
    }

    fn describe_walk(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        let sentence = if events_summary.runner_advancements.len() == 1 { // walk, no other runners
            format!("Ball four, {} walks.", prev_game_state_summary.batter)
        } else { // walk into home
            if let Some(runner_advancement) = events_summary.runner_advancements.iter().find(|runner_advancement| runner_advancement.to_base == Some(Base::Home)) {
                let part_1 = format!(
                    "Ball four, {} cycles the bases and walks in a run by {}",
                    prev_game_state_summary.batter,
                    runner_advancement.name,
                );
                let part_2 = format!(
                    "Now {} on first, {} on second, and {} on third.",
                    new_game_state_summary.bases.first.as_ref().unwrap(),
                    new_game_state_summary.bases.second.as_ref().unwrap(),
                    new_game_state_summary.bases.third.as_ref().unwrap(),
                );

                vec![part_1, part_2].join(" ")
            } else if new_game_state_summary.bases.are_loaded() { // walks bases loaded
                format!(
                    "Ball four, {} walks and {} has loaded the bases.",
                    prev_game_state_summary.batter,
                    prev_game_state_summary.pitcher,
                )
            } else { // walk with other runners
                let mut sentence = format!("Ball four, {} walks, ",  prev_game_state_summary.batter);
                let mut runner_parts = Vec::new();
                for runner_advancement in events_summary.runner_advancements.iter() {
                    if runner_advancement.from_base != Base::Batting {
                        let to_base = match runner_advancement.to_base.unwrap() {
                            Base::First => "first",
                            Base::Second => "second",
                            Base::Third => "third",
                            Base::Home => "home",
                            Base::Batting => unreachable!(),
                        };
                        runner_parts.push(format!("{} to {to_base}", runner_advancement.name));
                    }
                }
                sentence += &runner_parts.join(" and ");
                sentence += ".";
                sentence
            }
        };

        sentences.push(sentence);
    }

    fn describe_runner_advancements(
        &self,
        runner_advancements: &Vec<RunnerAdvancement>,
        sentences: &mut Vec<String>,
        skip_batter: bool,
    ) {
        let mut runner_parts = Vec::new();
        for runner_advancement in runner_advancements.iter() {
            if skip_batter && runner_advancement.from_base == Base::Batting {
                continue;
            }

            let part = match runner_advancement.to_base {
                Some(Base::First) => format!("{} to first", runner_advancement.name),
                Some(Base::Second) => format!("{} to second", runner_advancement.name),
                Some(Base::Third) => format!("{} to third", runner_advancement.name),
                Some(Base::Home) => format!("{} scores", runner_advancement.name),
                None => {
                    let end_base = match runner_advancement.from_base.next().unwrap() {
                        Base::First => "first",
                        Base::Second => "second",
                        Base::Third => "third",
                        Base::Home => "home plate",
                        Base::Batting => unreachable!(),
                    };
                    format!("{} is out at {}", runner_advancement.name, end_base)
                },
                Some(Base::Batting) => unreachable!(),
            };
            runner_parts.push(part);
        }

        let sentence = runner_parts.join(", ") + ".";
        sentences.push(sentence);
    }

    fn describe_base_hit(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        let hit_sentence = match events_summary.at_bat_outcome.as_ref().unwrap() {
            AtBatOutcome::Single => "That's a single to left field.".to_string(),
            AtBatOutcome::Double => "Hard hit to center field, extra base hit.".to_string(),
            AtBatOutcome::Triple => "Great swing to center field, and that'll be a triple.".to_string(),
            _ => unreachable!(),
        };
        sentences.push(hit_sentence);

        if events_summary.runner_advancements.len() > 1 {
            self.describe_runner_advancements(&events_summary.runner_advancements, sentences, true);
        }
    }

    fn describe_home_run(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        if events_summary.runner_advancements.len() == 1 {
            sentences.push("Home run!".to_string());
        } else if events_summary.runner_advancements.len() == 4 {
            sentences.push("Grand slam home run!".to_string());
        } else {
            let num_runs = events_summary.runner_advancements.len();
            let team_name = if prev_game_state_summary.half_inning.top { &self.visiting_team_name } else { &self.home_team_name };
            sentences.push(format!("Home run! And {} brings in {num_runs} runs for the {team_name}.", prev_game_state_summary.batter));
        }
    }

    fn describe_batter_out(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        let sentence = format!("He hits a ground ball to the left, and... he'll be out at first base.");
        sentences.push(sentence);

        if events_summary.runner_advancements.len() > 1 {
            self.describe_runner_advancements(&events_summary.runner_advancements, sentences, true);
        }
    }

    fn describe_pitch_with_at_bat_outcome(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
        sentences: &mut Vec<String>,
    ) {
        match events_summary.at_bat_outcome.as_ref().unwrap() {
            AtBatOutcome::Strikeout => self.describe_strikeout(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                sentences,
            ),
            AtBatOutcome::Walk => self.describe_walk(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                sentences,
            ),
            AtBatOutcome::Single | AtBatOutcome::Double | AtBatOutcome::Triple => self.describe_base_hit(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                sentences,
            ),
            AtBatOutcome::HomeRun => self.describe_home_run(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                sentences,
            ),
            AtBatOutcome::Out => self.describe_batter_out(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                sentences,
            ),
        }
    }

    pub fn describe_pitch_level_summaries(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summary: &EventsSummary,
        new_game_state_summary: &GameStateSummary,
    ) -> String {
        let mut sentences = Vec::new();

        // count description
        if prev_game_state_summary.count.is_empty() {
            sentences.push(format!(
                "{} pitching to {}.",
                prev_game_state_summary.pitcher,
                prev_game_state_summary.batter,
            ));
        } else if prev_game_state_summary.count.is_full() {
            sentences.push("It's a full count.".to_string());
        } else {
            sentences.push(format!(
                "Count is {}-{}.",
                prev_game_state_summary.count.balls,
                prev_game_state_summary.count.strikes,
            ));
        }

        // pitch description
        if events_summary.at_bat_outcome.is_none() {
            self.describe_pitch_with_no_at_bat_outcome(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                &mut sentences,
            );
        } else {
            self.describe_pitch_with_at_bat_outcome(
                prev_game_state_summary,
                events_summary,
                new_game_state_summary,
                &mut sentences,
            );
        }

        sentences.join(" ")
    }

    pub fn describe_half_inning_summaries(
        &self,
        prev_game_state_summary: &GameStateSummary,
        events_summaries: &Vec<EventsSummary>,
        game_state_summaries: &Vec<GameStateSummary>,
    ) -> String {
        let mut game_state_summaries = game_state_summaries.clone();
        game_state_summaries.insert(0, prev_game_state_summary.clone());

        let mut sentences = Vec::new();

        for i in 1..events_summaries.len() {
            let prev_game_state_summary = &game_state_summaries[i - 1];
            let events_summary = &events_summaries[i];
            let new_game_state_summary = &game_state_summaries[i];

            sentences.push(self.describe_pitch_level_summaries(
                &prev_game_state_summary,
                &events_summary,
                &new_game_state_summary,
            ));
        }

        sentences.join(" ")
    }
}
