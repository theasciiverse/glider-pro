//! Music.c: the music channel and the score it plays (the order of the
//! pieces, and the game's own shorter score that moving between rooms
//! prods and kicks).
//!
//! The channel is the audio layer's: StartMusic queues pieces on
//! `music_queue` (snd ids), StopTheMusic queues `MUSIC_STOP`, and when the
//! last queued piece ends the audio layer calls `music_piece_finished` (the
//! C's MusicCallBack) for the next one.

use super::*;

pub const BASE_BUFFER_MUSIC_ID: i32 = 2000;
pub const MAX_MUSIC: i32 = 7;
pub const LAST_MUSIC_PIECE: i32 = 16;
pub const LAST_GAME_PIECE: i32 = 6;
/// In `music_queue`: flush the music channel and quiet it.
pub const MUSIC_STOP: i32 = -1;

impl Game {
    //--------------------------------------------------------------  StartMusic

    /// Returns 0 (noErr), as the C's OSErr.
    pub fn start_music(&mut self) -> i32 {
        let the_err = 0;

        if self.dont_load_music {
            return the_err;
        }

        // UnivGetSoundVolume: the volume is not 0 when sound is on.
        if self.is_sound_on && !self.failed_music {
            // bufferCmd, then a nullCmd (param1 1964, no callback)
            self.music_queue
                .push(BASE_BUFFER_MUSIC_ID + self.music_sound_id);

            self.music_cursor += 1;
            if self.music_cursor >= LAST_MUSIC_PIECE {
                self.music_cursor = 0;
            }
            self.music_sound_id = self.music_score[self.music_cursor as usize];

            // bufferCmd, then callBackCmd: music_piece_finished follows it.
            self.music_queue
                .push(BASE_BUFFER_MUSIC_ID + self.music_sound_id);

            self.is_music_on = true;
        }

        the_err
    }

    //--------------------------------------------------------------  StopTheMusic

    pub fn stop_the_music(&mut self) {
        if self.dont_load_music {
            return;
        }

        if self.is_music_on && !self.failed_music {
            // flushCmd, quietCmd
            self.music_queue.push(MUSIC_STOP);

            self.is_music_on = false;
        }
    }

    //--------------------------------------------------------------  ToggleMusicWhilePlaying

    pub fn toggle_music_while_playing(&mut self) {
        if self.dont_load_music {
            return;
        }

        if self.is_play_music_game {
            if !self.is_music_on {
                let _the_err = self.start_music();
            }
        } else {
            if self.is_music_on {
                self.stop_the_music();
            }
        }
    }

    //--------------------------------------------------------------  SetMusicalPiece

    pub fn set_musical_mode(&mut self, new_mode: i32) {
        if self.dont_load_music {
            return;
        }

        match new_mode {
            KICK_GAME_SCORE_MODE => {
                self.music_cursor = 2;
            }

            PROD_GAME_SCORE_MODE => {
                self.music_cursor = -1;
            }

            _ => {
                self.music_mode = new_mode;
                self.music_cursor = 0;
            }
        }
    }

    //--------------------------------------------------------------  MusicCallBack

    /// MusicCallBack: the queued music has played out; picks the next
    /// piece and returns its snd id for the audio layer to play (its end
    /// calls this again). None when the music is off (the C's flush took
    /// the callback away).
    pub fn music_piece_finished(&mut self) -> Option<i32> {
        if !self.is_music_on || self.failed_music || self.dont_load_music {
            return None;
        }

        match self.music_mode {
            PLAY_GAME_SCORE_MODE => {
                self.music_cursor += 1;
                if self.music_cursor >= LAST_GAME_PIECE {
                    self.music_cursor = 1;
                }
                self.music_sound_id = self.game_score[self.music_cursor as usize];
                if self.music_sound_id < 0 {
                    self.music_cursor += self.music_sound_id;
                    self.music_sound_id = self.game_score[self.music_cursor as usize];
                }
            }

            PLAY_WHOLE_SCORE_MODE => {
                self.music_cursor += 1;
                if self.music_cursor >= LAST_MUSIC_PIECE - 1 {
                    self.music_cursor = 0;
                }
                self.music_sound_id = self.music_score[self.music_cursor as usize];
            }

            _ => {
                self.music_sound_id = self.music_mode;
            }
        }

        Some(BASE_BUFFER_MUSIC_ID + self.music_sound_id)
    }

    //--------------------------------------------------------------  InitMusic

    /// The pieces themselves are the audio layer's (LoadMusicSounds,
    /// OpenMusicChannel).
    pub fn init_music(&mut self) {
        if self.dont_load_music {
            return;
        }

        self.failed_music = false;
        self.is_music_on = false;

        self.music_score[0] = 0;
        self.music_score[1] = 1;
        self.music_score[2] = 2;
        self.music_score[3] = 3;
        self.music_score[4] = 4;
        self.music_score[5] = 4;
        self.music_score[6] = 0;
        self.music_score[7] = 1;
        self.music_score[8] = 2;
        self.music_score[9] = 3;
        self.music_score[10] = PLAY_CHORUS;
        self.music_score[11] = PLAY_CHORUS;
        self.music_score[12] = PLAY_REFRAIN_SPARSE1;
        self.music_score[13] = PLAY_REFRAIN_SPARSE2;
        self.music_score[14] = PLAY_CHORUS;
        self.music_score[15] = PLAY_CHORUS;

        self.game_score[0] = PLAY_REFRAIN_SPARSE2;
        self.game_score[1] = PLAY_REFRAIN_SPARSE1;
        self.game_score[2] = -1;
        self.game_score[3] = PLAY_REFRAIN_SPARSE2;
        self.game_score[4] = PLAY_CHORUS;
        self.game_score[5] = PLAY_CHORUS;

        self.music_cursor = 0;
        self.music_sound_id = self.music_score[self.music_cursor as usize];
        self.music_mode = PLAY_WHOLE_SCORE_MODE;

        if self.is_play_music_idle {
            let the_err = self.start_music();
            if the_err != 0 {
                log::warn!("YellowAlert kYellowNoMusic {the_err}");
                self.failed_music = true;
            }
        }
    }

    //--------------------------------------------------------------  KillMusic

    pub fn kill_music(&mut self) {
        if self.dont_load_music {
            return;
        }

        self.stop_the_music();
    }
}
