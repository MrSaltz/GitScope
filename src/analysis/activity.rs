use std::collections::BTreeMap;

use chrono::Datelike;

use crate::model::{ActivityStats, CommitRecord, MonthActivity};

#[derive(Debug, Default)]
pub struct ActivityAccumulator {
    months: BTreeMap<(i32, u32), usize>,
}

impl ActivityAccumulator {
    pub fn add(&mut self, commit: &CommitRecord) {
        let key = (commit.date.year(), commit.date.month());
        *self.months.entry(key).or_default() += 1;
    }

    pub fn finish(self) -> ActivityStats {
        let (Some(&first), Some(&last)) =
            (self.months.keys().next(), self.months.keys().next_back())
        else {
            return ActivityStats {
                by_month: Vec::new(),
            };
        };

        let mut by_month = Vec::new();
        let (mut year, mut month) = first;
        while (year, month) <= last {
            by_month.push(MonthActivity {
                month: format!("{year:04}-{month:02}"),
                commits: self.months.get(&(year, month)).copied().unwrap_or(0),
            });
            if month == 12 {
                year += 1;
                month = 1;
            } else {
                month += 1;
            }
        }
        ActivityStats { by_month }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::testutil::record;

    fn months(whens: &[&str]) -> Vec<(String, usize)> {
        let mut acc = ActivityAccumulator::default();
        for w in whens {
            acc.add(&record("A", "a@x.io", w));
        }
        acc.finish()
            .by_month
            .into_iter()
            .map(|m| (m.month, m.commits))
            .collect()
    }

    #[test]
    fn no_commits_no_months() {
        assert!(months(&[]).is_empty());
    }

    #[test]
    fn groups_by_month_in_chronological_order() {
        let got = months(&["2026-02-10 10:00", "2026-01-03 10:00", "2026-02-11 10:00"]);
        assert_eq!(got, vec![("2026-01".into(), 1), ("2026-02".into(), 2)]);
    }

    #[test]
    fn fills_gaps_with_zero_and_crosses_year_boundaries() {
        let got = months(&["2025-11-05 10:00", "2026-02-01 10:00"]);
        assert_eq!(
            got,
            vec![
                ("2025-11".into(), 1),
                ("2025-12".into(), 0),
                ("2026-01".into(), 0),
                ("2026-02".into(), 1),
            ]
        );
    }
}
