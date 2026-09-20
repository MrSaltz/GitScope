use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};

use crate::model::{CommitRecord, CommitStats, HourCount, WeekdayCount};

const WEEKDAYS: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

#[derive(Debug, Default)]
pub struct CommitsAccumulator {
    total: usize,
    first: Option<DateTime<Utc>>,
    last: Option<DateTime<Utc>>,
    first_message: Option<String>,
    last_message: Option<String>,
    insertions: usize,
    deletions: usize,
    by_weekday: [usize; 7],
    by_hour: [usize; 24],
}

impl CommitsAccumulator {
    pub fn add(&mut self, commit: &CommitRecord) {
        self.total += 1;
        // Empate de data: o percurso vem do mais novo ao mais antigo, então `<=` deixa
        // o mais antigo como "primeiro" e `>` deixa o mais novo como "último".
        if self.first.is_none_or(|d| commit.date <= d) {
            self.first = Some(commit.date);
            self.first_message = Some(commit.message.clone());
        }
        if self.last.is_none_or(|d| commit.date > d) {
            self.last = Some(commit.date);
            self.last_message = Some(commit.message.clone());
        }
        self.insertions += commit.changes.insertions;
        self.deletions += commit.changes.deletions;
        self.by_weekday[commit.date.weekday().num_days_from_monday() as usize] += 1;
        self.by_hour[commit.date.hour() as usize] += 1;
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn finish(self) -> CommitStats {
        let most_active_hour = (self.total > 0)
            .then(|| {
                self.by_hour
                    .iter()
                    .enumerate()
                    .max_by(|(ha, ca), (hb, cb)| ca.cmp(cb).then_with(|| hb.cmp(ha)))
                    .map(|(hour, _)| hour as u32)
            })
            .flatten();

        CommitStats {
            total: self.total,
            first: self.first,
            last: self.last,
            first_message: self.first_message,
            last_message: self.last_message,
            insertions: self.insertions,
            deletions: self.deletions,
            most_active_hour,
            by_weekday: WEEKDAYS
                .iter()
                .zip(self.by_weekday)
                .map(|(&weekday, commits)| WeekdayCount { weekday, commits })
                .collect(),
            by_hour: self
                .by_hour
                .iter()
                .enumerate()
                .map(|(hour, &commits)| HourCount {
                    hour: hour as u32,
                    commits,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::testutil::record;

    fn stats(whens: &[&str]) -> CommitStats {
        let mut acc = CommitsAccumulator::default();
        for w in whens {
            acc.add(&record("A", "a@x.io", w));
        }
        acc.finish()
    }

    #[test]
    fn empty_history_has_no_dates_or_active_hour() {
        let s = stats(&[]);
        assert_eq!(s.total, 0);
        assert_eq!(s.first, None);
        assert_eq!(s.last, None);
        assert_eq!(s.most_active_hour, None);
        assert_eq!(s.by_weekday.len(), 7);
        assert_eq!(s.by_hour.len(), 24);
    }

    #[test]
    fn tracks_first_and_last_regardless_of_input_order() {
        let s = stats(&["2026-03-01 10:00", "2026-01-03 12:30", "2026-09-18 14:32"]);
        assert_eq!(s.total, 3);
        assert_eq!(s.first.unwrap().to_rfc3339(), "2026-01-03T12:30:00+00:00");
        assert_eq!(s.last.unwrap().to_rfc3339(), "2026-09-18T14:32:00+00:00");
    }

    #[test]
    fn keeps_the_messages_of_the_first_and_last_commit() {
        use crate::analysis::testutil::record_with;
        let mut oldest = record_with("A", "a@x.io", "2026-01-01 10:00", &["a"], 5, 1);
        oldest.message = "initial".into();
        let mut newest = record_with("A", "a@x.io", "2026-03-01 10:00", &["a", "b"], 10, 2);
        newest.message = "latest".into();
        let mut middle = record("A", "a@x.io", "2026-02-01 10:00");
        middle.message = "middle".into();

        let mut acc = CommitsAccumulator::default();
        for c in [&newest, &oldest, &middle] {
            acc.add(c);
        }
        let s = acc.finish();
        assert_eq!(s.first_message.as_deref(), Some("initial"));
        assert_eq!(s.last_message.as_deref(), Some("latest"));
        assert_eq!((s.insertions, s.deletions), (15, 3));
    }

    #[test]
    fn same_instant_ties_prefer_the_older_walk_position_for_first_and_newer_for_last() {
        let mut walked_first = record("A", "a@x.io", "2026-01-01 10:00");
        walked_first.message = "walked first".into();
        let mut walked_second = record("A", "a@x.io", "2026-01-01 10:00");
        walked_second.message = "walked second".into();

        let mut acc = CommitsAccumulator::default();
        acc.add(&walked_first);
        acc.add(&walked_second);
        let s = acc.finish();
        assert_eq!(s.first_message.as_deref(), Some("walked second"));
        assert_eq!(s.last_message.as_deref(), Some("walked first"));
    }

    #[test]
    fn empty_history_has_no_messages_and_zero_lines() {
        let s = stats(&[]);
        assert_eq!(s.first_message, None);
        assert_eq!(s.last_message, None);
        assert_eq!((s.insertions, s.deletions), (0, 0));
    }

    #[test]
    fn buckets_by_weekday_starting_on_monday() {
        let s = stats(&["2026-01-05 09:00", "2026-01-05 10:00", "2026-01-11 09:00"]);
        assert_eq!(s.by_weekday[0].weekday, Weekday::Mon);
        assert_eq!(s.by_weekday[0].commits, 2);
        assert_eq!(s.by_weekday[6].weekday, Weekday::Sun);
        assert_eq!(s.by_weekday[6].commits, 1);
        assert_eq!(s.by_weekday.iter().map(|d| d.commits).sum::<usize>(), 3);
    }

    #[test]
    fn most_active_hour_prefers_highest_then_earliest() {
        let s = stats(&["2026-01-05 14:10", "2026-01-06 14:50", "2026-01-07 09:00"]);
        assert_eq!(s.most_active_hour, Some(14));
        assert_eq!(s.by_hour[14].commits, 2);
        assert_eq!(s.by_hour[9].commits, 1);

        let tie = stats(&["2026-01-05 18:00", "2026-01-06 07:00"]);
        assert_eq!(tie.most_active_hour, Some(7));
    }
}
