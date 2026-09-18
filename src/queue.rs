use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Repeat {
    #[default]
    Off,
    All,
    One,
}

impl Repeat {
    pub fn cycle(self) -> Self {
        match self {
            Self::Off => Self::All,
            Self::All => Self::One,
            Self::One => Self::Off,
        }
    }
}

/// Queue entries are library indices. Position identifies an occurrence, so a
/// song added twice remains two independently removable entries.
#[derive(Default)]
pub struct Queue {
    pub entries: Vec<usize>,
    pub position: Option<usize>,
    pub shuffle: bool,
    pub repeat: Repeat,
    history: Vec<usize>,
    remaining: Vec<usize>,
    seed: u64,
}

impl Queue {
    pub fn current(&self) -> Option<usize> {
        self.position.and_then(|p| self.entries.get(p).copied())
    }
    pub fn replace(&mut self, entries: Vec<usize>, at: usize) {
        self.entries = entries;
        self.position = (at < self.entries.len()).then_some(at);
        self.history.clear();
        self.reset_shuffle();
    }
    pub fn reset_shuffle(&mut self) {
        self.remaining = (0..self.entries.len())
            .filter(|i| Some(*i) != self.position)
            .collect();
    }
    pub fn append(&mut self, track: usize) {
        self.remaining.push(self.entries.len());
        self.entries.push(track);
    }
    pub fn select(&mut self, position: usize) {
        if position < self.entries.len() {
            if let Some(old) = self.position {
                self.history.push(old);
            }
            self.position = Some(position);
            self.remaining.retain(|p| *p != position);
        }
    }
    pub fn next(&mut self, automatic: bool) -> Option<usize> {
        let old = self.position?;
        if automatic && self.repeat == Repeat::One {
            return self.current();
        }
        let next = if self.shuffle {
            if self.remaining.is_empty() {
                if self.repeat == Repeat::Off {
                    return None;
                }
                self.reset_shuffle();
            }
            if self.remaining.is_empty() {
                old
            } else {
                if self.seed == 0 {
                    self.seed = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64
                        | 1;
                }
                self.seed ^= self.seed << 13;
                self.seed ^= self.seed >> 7;
                self.seed ^= self.seed << 17;
                self.remaining
                    .swap_remove(self.seed as usize % self.remaining.len())
            }
        } else if old + 1 < self.entries.len() {
            old + 1
        } else if self.repeat != Repeat::Off {
            0
        } else {
            return None;
        };
        self.history.push(old);
        self.position = Some(next);
        self.current()
    }
    pub fn previous(&mut self) -> Option<usize> {
        let old = self.position?;
        let previous = self.history.pop().unwrap_or_else(|| old.saturating_sub(1));
        self.position = Some(previous);
        if self.shuffle && previous != old {
            self.remaining.retain(|p| *p != previous);
            if !self.remaining.contains(&old) {
                self.remaining.push(old);
            }
        }
        self.current()
    }
    pub fn remove(&mut self, at: usize) -> bool {
        if at >= self.entries.len() {
            return false;
        }
        let was_current = self.position == Some(at);
        self.entries.remove(at);
        self.position = self.position.and_then(|p| {
            if self.entries.is_empty() {
                None
            } else if p > at {
                Some(p - 1)
            } else {
                Some(p.min(self.entries.len() - 1))
            }
        });
        self.history.clear();
        self.reset_shuffle();
        was_current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn end_and_repeat_have_distinct_behaviour() {
        let mut q = Queue::default();
        q.replace(vec![10, 20], 1);
        assert_eq!(q.next(true), None);
        q.repeat = Repeat::One;
        assert_eq!(q.next(true), Some(20));
        assert_eq!(q.next(false), Some(10));
        q.repeat = Repeat::All;
        assert_eq!(q.next(true), Some(20));
        assert_eq!(q.next(true), Some(10));
    }
    #[test]
    fn shuffle_visits_each_occurrence_once_and_previous_retraces() {
        let mut q = Queue {
            shuffle: true,
            seed: 42,
            ..Queue::default()
        };
        q.replace(vec![0, 1, 2, 3], 0);
        let mut played = vec![0];
        while let Some(track) = q.next(true) {
            played.push(track);
        }
        let prior = played[2];
        played.sort();
        assert_eq!(played, vec![0, 1, 2, 3]);
        assert_eq!(q.previous(), Some(prior));
    }
    #[test]
    fn removal_preserves_current_occurrence() {
        let mut q = Queue::default();
        q.replace(vec![8, 8, 9], 1);
        assert!(!q.remove(0));
        assert_eq!(q.current(), Some(8));
        assert!(q.remove(0));
        assert_eq!(q.current(), Some(9));
        assert!(q.remove(0));
        assert_eq!(q.current(), None);
    }
}
