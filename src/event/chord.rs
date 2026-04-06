use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ChordState {
    pub pending_keys: Vec<KeyEvent>,
    pub last_key_time: Option<Instant>,
    pub timeout: Duration,
}

pub enum ChordResult {
    /// Chord fully matched something, emit this key
    Matched(KeyEvent),
    /// Chord is pending, waiting for more keys. Do not process anything yet.
    Pending,
    /// Chord broken, these are the keys that should be processed normally (the buffered ones + the current one).
    Failed(Vec<KeyEvent>),
    /// Not part of any chord, process normally
    None,
}

impl Default for ChordState {
    fn default() -> Self {
        Self {
            pending_keys: Vec::new(),
            last_key_time: None,
            timeout: Duration::from_millis(300),
        }
    }
}

impl ChordState {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            pending_keys: Vec::new(),
            last_key_time: None,
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    pub fn clear(&mut self) {
        self.pending_keys.clear();
        self.last_key_time = None;
    }

    pub fn handle_key(&mut self, key: KeyEvent, is_insert_mode: bool) -> ChordResult {
        if !is_insert_mode {
            let pending = std::mem::take(&mut self.pending_keys);
            self.last_key_time = None;
            if pending.is_empty() {
                return ChordResult::None;
            } else {
                let mut failed = pending;
                failed.push(key);
                return ChordResult::Failed(failed);
            }
        }

        // Check timeout
        if let Some(last_time) = self.last_key_time {
            if last_time.elapsed() > self.timeout {
                let mut failed = std::mem::take(&mut self.pending_keys);
                self.last_key_time = None;
                failed.push(key);
                return ChordResult::Failed(failed);
            }
        }

        // Only simple characters without modifiers (or with Shift) can be part of chords we support here
        let _c = match key.code {
            KeyCode::Char(c)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                c
            }
            _ => {
                if self.pending_keys.is_empty() {
                    return ChordResult::None;
                } else {
                    let mut failed = std::mem::take(&mut self.pending_keys);
                    self.last_key_time = None;
                    failed.push(key);
                    return ChordResult::Failed(failed);
                }
            }
        };

        self.pending_keys.push(key);
        self.last_key_time = Some(Instant::now());

        let s: String = self
            .pending_keys
            .iter()
            .filter_map(|k| {
                if let KeyCode::Char(c) = k.code {
                    Some(c)
                } else {
                    None
                }
            })
            .collect();

        // Hardcoded escapes for now: "jk"
        if s == "jk" {
            self.clear();
            return ChordResult::Matched(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
        }

        // Check if prefix of any chord
        if "jk".starts_with(&s) {
            return ChordResult::Pending;
        }

        let failed = std::mem::take(&mut self.pending_keys);
        self.last_key_time = None;
        ChordResult::Failed(failed)
    }

    pub fn check_timeout(&mut self) -> Option<Vec<KeyEvent>> {
        if !self.pending_keys.is_empty() {
            if let Some(last_time) = self.last_key_time {
                if last_time.elapsed() > self.timeout {
                    let failed = std::mem::take(&mut self.pending_keys);
                    self.last_key_time = None;
                    return Some(failed);
                }
            }
        }
        None
    }
}
