use crate::baseball::{Count, EventsSummary, GameStateSummary};

pub enum Granularity {
    Pitch,
    Inning,
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
            Self::Inning
        } else {
            Self::Pitch
        }
    }
}

fn describe_pitch_with_no_at_bat_outcome(
    prev_game_state_summary: &GameStateSummary,
    events_summary: &EventsSummary,
    new_game_state_summary: &GameStateSummary,
    granularity: &Granularity,
) -> String {
    let mut sentences = Vec::new();

    sentences.push(format!(
        "{} pitches to {}.",
        prev_game_state_summary.pitcher,
        prev_game_state_summary.batter,
    ));

    if prev_game_state_summary.count.is_full() {
        sentences.push("It's a full count.".to_string());
    } else if !prev_game_state_summary.count.is_empty() {
        sentences.push(format!(
            "Count is {}-{}.",
            prev_game_state_summary.count.balls,
            prev_game_state_summary.count.strikes,
        ));
    }

    sentences.join(" ")
}

pub fn describe_summaries(
    prev_game_state_summary: &GameStateSummary,
    events_summary: &EventsSummary,
    new_game_state_summary: &GameStateSummary,
    granularity: &Granularity,
) -> String {
    if events_summary.at_bat_outcome.is_none() {
        describe_pitch_with_no_at_bat_outcome(prev_game_state_summary, events_summary, new_game_state_summary, granularity)
    } else {
        todo!()
    }
}
