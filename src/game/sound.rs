//! Sound.c: three sound channels, each playing one sound at a time, and
//! the priorities that decide which channel a new sound takes.
//!
//! The Sound Manager is the audio layer's: where the C starts a sound on a
//! channel, `(sound, channel)` goes onto `sound_queue`; when a channel's
//! sound ends, the audio layer calls `sound_finished(channel)` (the C's
//! channel callbacks).

use super::*;

/// The `snd ` id of sound 0; sound n is `BASE_BUFFER_SOUND_ID + n`.
pub const BASE_BUFFER_SOUND_ID: i32 = 1000;
pub const MAX_SOUNDS: i32 = 64;
pub const NO_SOUND_PLAYING: i32 = -1;

impl Game {
    //--------------------------------------------------------------  PlayPrioritySound

    pub fn play_priority_sound(&mut self, which: i32, priority: i32) {
        if self.failed_sound || self.dont_load_sounds {
            return;
        }

        if (priority == TRIGGER_PRIORITY)
            && ((self.priority0 == TRIGGER_PRIORITY)
                || (self.priority1 == TRIGGER_PRIORITY)
                || (self.priority2 == TRIGGER_PRIORITY))
        {
            return;
        }

        let mut whos_lowest = 0;
        let mut lowest_priority = self.priority0;

        if self.priority1 < lowest_priority {
            lowest_priority = self.priority1;
            whos_lowest = 1;
        }

        if self.priority2 < lowest_priority {
            lowest_priority = self.priority2;
            whos_lowest = 2;
        }

        if priority >= lowest_priority {
            match whos_lowest {
                0 => self.play_sound0(which, priority),
                1 => self.play_sound1(which, priority),
                2 => self.play_sound2(which, priority),
                _ => {}
            }
        }
    }

    /// The name the shared brief used for PlayPrioritySound.
    pub fn play_prioritized_sound(&mut self, which: i32, priority: i32) {
        self.play_priority_sound(which, priority);
    }

    //--------------------------------------------------------------  FlushAnyTriggerPlaying

    pub fn flush_any_trigger_playing(&mut self) {
        // quietCmd + flushCmd on the channel: the audio layer takes the
        // stop sound as silence.
        if self.priority0 == TRIGGER_PRIORITY {
            self.sound_queue.push((NO_SOUND_PLAYING, 0));
        }

        if self.priority1 == TRIGGER_PRIORITY {
            self.sound_queue.push((NO_SOUND_PLAYING, 1));
        }

        if self.priority2 == TRIGGER_PRIORITY {
            self.sound_queue.push((NO_SOUND_PLAYING, 2));
        }
    }

    //--------------------------------------------------------------  PlaySound0

    pub fn play_sound0(&mut self, sound_id: i32, priority: i32) {
        if self.failed_sound || self.dont_load_sounds {
            return;
        }

        if self.is_sound_on {
            self.priority0 = priority;
            self.sound_playing0 = sound_id;

            self.sound_queue.push((sound_id, 0));
        }
    }

    //--------------------------------------------------------------  PlaySound1

    pub fn play_sound1(&mut self, sound_id: i32, priority: i32) {
        if self.failed_sound || self.dont_load_sounds {
            return;
        }

        if self.is_sound_on {
            self.priority1 = priority;
            self.sound_playing1 = sound_id;

            self.sound_queue.push((sound_id, 1));
        }
    }

    //--------------------------------------------------------------  PlaySound2

    pub fn play_sound2(&mut self, sound_id: i32, priority: i32) {
        if self.failed_sound || self.dont_load_sounds {
            return;
        }

        if self.is_sound_on {
            self.sound_queue.push((sound_id, 2));

            self.priority2 = priority;
            self.sound_playing2 = sound_id;
        }
    }

    //--------------------------------------------------------------  CallBack0/1/2

    /// The channel callbacks (CallBack0, CallBack1, CallBack2): the sound
    /// on `channel` has finished.
    pub fn sound_finished(&mut self, channel: i32) {
        match channel {
            0 => {
                self.priority0 = 0;
                self.sound_playing0 = NO_SOUND_PLAYING;
            }
            1 => {
                self.priority1 = 0;
                self.sound_playing1 = NO_SOUND_PLAYING;
            }
            2 => {
                self.priority2 = 0;
                self.sound_playing2 = NO_SOUND_PLAYING;
            }
            _ => {}
        }
    }

    //--------------------------------------------------------------  LoadTriggerSound

    /// Returns 0 (noErr) or -1, as the C's OSErr.
    pub fn load_trigger_sound(&mut self, sound_id: i32) -> i32 {
        let the_err;

        if self.dont_load_sounds || self.trigger_sound_id.is_some() {
            the_err = -1;
        } else {
            //		FlushAnyTriggerPlaying();

            let found = self
                .assets
                .as_ref()
                .and_then(|a| a.sound(Some(&self.house), sound_id as i16))
                .is_some();
            if !found {
                the_err = -1;
            } else {
                self.trigger_sound_id = Some(sound_id);
                the_err = 0;
            }
        }

        the_err
    }

    //--------------------------------------------------------------  DumpTriggerSound

    pub fn dump_trigger_sound(&mut self) {
        self.trigger_sound_id = None;
    }

    //--------------------------------------------------------------  InitSound

    /// The sounds themselves are the audio layer's (LoadBufferSounds,
    /// OpenSoundChannels): this sets up the channels' state.
    pub fn init_sound(&mut self) {
        if self.dont_load_sounds {
            return;
        }

        self.failed_sound = false;

        self.priority0 = 0;
        self.priority1 = 0;
        self.priority2 = 0;
        self.sound_playing0 = NO_SOUND_PLAYING;
        self.sound_playing1 = NO_SOUND_PLAYING;
        self.sound_playing2 = NO_SOUND_PLAYING;

        self.trigger_sound_id = None;
    }

    //--------------------------------------------------------------  KillSound

    pub fn kill_sound(&mut self) {
        if self.dont_load_sounds {
            return;
        }

        self.dump_trigger_sound();
    }
}
