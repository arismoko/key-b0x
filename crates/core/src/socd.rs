use crate::config::SocdMode;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SocdPairState {
    was_dir1: bool,
    was_dir2: bool,
    lock_dir1: bool,
    lock_dir2: bool,
}

impl SocdPairState {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

pub(crate) fn resolve_pair(
    mode: SocdMode,
    dir1_pressed: bool,
    dir2_pressed: bool,
    state: &mut SocdPairState,
) -> (bool, bool) {
    match mode {
        SocdMode::Neutral => resolve_neutral(dir1_pressed, dir2_pressed),
        SocdMode::SecondInputPriority => {
            resolve_second_input_priority(dir1_pressed, dir2_pressed, state)
        }
        SocdMode::SecondInputPriorityNoReactivation => {
            resolve_second_input_priority_no_reactivation(dir1_pressed, dir2_pressed, state)
        }
        SocdMode::Dir1Priority => resolve_dir1_priority(dir1_pressed, dir2_pressed),
        SocdMode::Dir2Priority => {
            let (dir2, dir1) = resolve_dir1_priority(dir2_pressed, dir1_pressed);
            (dir1, dir2)
        }
    }
}

fn resolve_neutral(dir1_pressed: bool, dir2_pressed: bool) -> (bool, bool) {
    if dir1_pressed && dir2_pressed {
        return (false, false);
    }

    (dir1_pressed, dir2_pressed)
}

fn resolve_dir1_priority(dir1_pressed: bool, dir2_pressed: bool) -> (bool, bool) {
    if dir1_pressed && dir2_pressed {
        return (true, false);
    }

    (dir1_pressed, dir2_pressed)
}

fn resolve_second_input_priority(
    dir1_pressed: bool,
    dir2_pressed: bool,
    state: &mut SocdPairState,
) -> (bool, bool) {
    let mut result_dir1 = false;
    let mut result_dir2 = false;

    if dir1_pressed && state.was_dir2 {
        result_dir1 = true;
        result_dir2 = false;
    }
    if dir2_pressed && state.was_dir1 {
        result_dir1 = false;
        result_dir2 = true;
    }
    if !dir1_pressed && dir2_pressed {
        result_dir1 = false;
        result_dir2 = true;
        state.was_dir2 = true;
        state.was_dir1 = false;
    }
    if dir1_pressed && !dir2_pressed {
        result_dir1 = true;
        result_dir2 = false;
        state.was_dir1 = true;
        state.was_dir2 = false;
    }
    if !dir1_pressed && !dir2_pressed {
        state.reset();
    }

    (result_dir1, result_dir2)
}

fn resolve_second_input_priority_no_reactivation(
    dir1_pressed: bool,
    dir2_pressed: bool,
    state: &mut SocdPairState,
) -> (bool, bool) {
    let mut result_dir1 = false;
    let mut result_dir2 = false;

    if dir1_pressed && dir2_pressed {
        if state.was_dir2 {
            result_dir1 = true;
            result_dir2 = false;
            state.lock_dir2 = true;
        }
        if state.was_dir1 {
            result_dir1 = false;
            result_dir2 = true;
            state.lock_dir1 = true;
        }
    }
    if !dir1_pressed && dir2_pressed && !state.lock_dir2 {
        result_dir1 = false;
        result_dir2 = true;
        state.was_dir2 = true;
        state.was_dir1 = false;
        state.lock_dir1 = false;
    }
    if dir1_pressed && !dir2_pressed && !state.lock_dir1 {
        result_dir1 = true;
        result_dir2 = false;
        state.was_dir1 = true;
        state.was_dir2 = false;
        state.lock_dir2 = false;
    }
    if !dir1_pressed && !dir2_pressed {
        state.reset();
    }

    (result_dir1, result_dir2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_input_priority_reactivates_first_direction() {
        let mut state = SocdPairState::default();

        let (left, right) = resolve_pair(SocdMode::SecondInputPriority, false, true, &mut state);
        assert_eq!((left, right), (false, true));

        let (left, right) = resolve_pair(SocdMode::SecondInputPriority, true, true, &mut state);
        assert_eq!((left, right), (true, false));

        let (left, right) = resolve_pair(SocdMode::SecondInputPriority, false, true, &mut state);
        assert_eq!((left, right), (false, true));
    }

    #[test]
    fn second_input_priority_no_reactivation_requires_fresh_press() {
        let mut state = SocdPairState::default();

        let (left, right) = resolve_pair(
            SocdMode::SecondInputPriorityNoReactivation,
            false,
            true,
            &mut state,
        );
        assert_eq!((left, right), (false, true));

        let (left, right) = resolve_pair(
            SocdMode::SecondInputPriorityNoReactivation,
            true,
            true,
            &mut state,
        );
        assert_eq!((left, right), (true, false));

        let (left, right) = resolve_pair(
            SocdMode::SecondInputPriorityNoReactivation,
            false,
            true,
            &mut state,
        );
        assert_eq!((left, right), (false, false));
    }

    #[test]
    fn neutral_cleans_opposites_to_neutral() {
        let mut state = SocdPairState::default();
        let resolved = resolve_pair(SocdMode::Neutral, true, true, &mut state);
        assert_eq!(resolved, (false, false));
    }

    #[test]
    fn directional_priority_prefers_configured_side() {
        let mut state = SocdPairState::default();
        let resolved = resolve_pair(SocdMode::Dir1Priority, true, true, &mut state);
        assert_eq!(resolved, (true, false));

        let mut state = SocdPairState::default();
        let resolved = resolve_pair(SocdMode::Dir2Priority, true, true, &mut state);
        assert_eq!(resolved, (false, true));
    }
}
