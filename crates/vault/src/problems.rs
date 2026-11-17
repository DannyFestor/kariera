use std::cell::RefCell;

use crate::Problem;

/// A value that could not be read. Its problems are already in the note's `Problems`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Reported;

/// Every problem of one note. A note parser reads each field into its own binding before it combines them with
/// `?`, so one run reports every bad field of a note instead of only the first. The field and section readers
/// share the collector, hence the `RefCell`.
#[derive(Debug, Default)]
pub(crate) struct Problems(RefCell<Vec<Problem>>);

impl Problems {
    pub(crate) fn record(&self, problem: Problem) {
        self.0.borrow_mut().push(problem);
    }

    pub(crate) fn adopt<T>(&self, result: Result<T, Vec<Problem>>) -> Result<T, Reported> {
        result.map_err(|problems| {
            self.0.borrow_mut().extend(problems);
            Reported
        })
    }

    pub(crate) fn into_result<T>(self, value: Result<T, Reported>) -> Result<T, Vec<Problem>> {
        let problems = self.0.into_inner();
        match value {
            Ok(value) if problems.is_empty() => Ok(value),
            Ok(_) | Err(Reported) => Err(problems),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Problems, Reported};
    use crate::{Location, Problem};

    fn problem(message: &str) -> Problem {
        Problem::new(Location::Note, None, message)
    }

    #[test]
    fn a_value_without_problems_is_the_result() {
        assert_eq!(Problems::default().into_result(Ok(5)), Ok(5));
    }

    #[test]
    fn any_recorded_problem_fails_the_result() {
        let problems = Problems::default();
        problems.record(problem("first"));

        assert_eq!(problems.into_result(Ok(5)), Err(vec![problem("first")]));
    }

    #[test]
    fn adopted_problems_are_kept_in_order() {
        let problems = Problems::default();
        problems.record(problem("first"));

        let adopted = problems.adopt::<u8>(Err(vec![problem("second"), problem("third")]));

        assert_eq!(adopted, Err(Reported));
        assert_eq!(
            problems.into_result(adopted),
            Err(vec![problem("first"), problem("second"), problem("third")])
        );
    }

    #[test]
    fn an_adopted_value_passes_through() {
        let problems = Problems::default();

        assert_eq!(problems.adopt(Ok::<_, Vec<Problem>>(5)), Ok(5));
    }
}
