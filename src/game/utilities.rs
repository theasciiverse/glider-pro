//! Utilities.c: the random numbers and the square root play needs
//! (DelayTicks and WaitForInputEvent are in `mod.rs`; the rest of the file
//! is Mac set-up and is left out).

use super::*;

impl Game {
    /// The Toolbox's Random: QuickDraw's `randSeed` times 16807 modulo
    /// 2^31 - 1, its low word as a signed integer (0x8000 gives 0).
    pub fn toolbox_random(&mut self) -> i32 {
        if self.rand_seed == 0 {
            self.rand_seed = 1; // QuickDraw starts randSeed at 1
        }
        self.rand_seed = ((self.rand_seed as u64 * 16807) % 0x7FFF_FFFF) as u32;
        let r = self.rand_seed as u16 as i16;
        if r == i16::MIN { 0 } else { r as i32 }
    }

    //--------------------------------------------------------------  RandomInt
    // Returns a random integer (short) within "range".

    pub fn random_int(&mut self, range: i32) -> i32 {
        let mut raw_result = self.toolbox_random() as i64;
        if raw_result < 0 {
            raw_result *= -1;
        }
        raw_result = (raw_result * range as i16 as i64) / 32768;

        raw_result as i16 as i32
    }

    //--------------------------------------------------------------  RandomLong
    // Returns a random long interger within "range".

    pub fn random_long(&mut self, range: i32) -> i32 {
        let high_word = ((range as u32 & 0xFFFF0000) >> 16) as i64;
        let low_word = (range & 0x0000FFFF) as i64;

        let mut raw_result_hi = self.toolbox_random() as i64;
        if raw_result_hi < 0 {
            raw_result_hi *= -1;
        }
        raw_result_hi = (raw_result_hi * high_word) / 32768;

        let mut raw_result_lo = self.toolbox_random() as i64;
        if raw_result_lo < 0 {
            raw_result_lo *= -1;
        }
        raw_result_lo = (raw_result_lo * low_word) / 32768;

        raw_result_hi = (raw_result_hi << 16) + raw_result_lo;

        raw_result_hi as i32
    }

    //--------------------------------------------------------------  InitRandomLongQUS

    // Initializes random seed for quick & dirty long random number function (below).

    pub fn init_random_long_qus(&mut self) {
        self.the_seed = get_date_time();
    }

    //--------------------------------------------------------------  RandomLongQUS

    // Very simple (but fast) pseudo-random number generator.

    pub fn random_long_qus(&mut self) -> u32 {
        self.the_seed = self.the_seed.wrapping_mul(1103515245).wrapping_add(12345);
        self.the_seed
    }

    //--------------------------------------------------------------  LongSquareRoot

    // This is a quick and dirty square root function that returns pretty
    // accurate long integer results.  It uses no transcendental functions or
    // floating point.

    pub fn long_square_root(&self, the_number: i32) -> i32 {
        long_square_root(the_number)
    }
}

/// LongSquareRoot, as a plain function.
pub fn long_square_root(the_number: i32) -> i32 {
    if the_number <= 1 {
        return the_number;
    }

    let mut next_trial = the_number / 2;
    let mut current_answer;

    loop {
        current_answer = next_trial;
        next_trial = (next_trial + the_number / next_trial) / 2;
        if next_trial >= current_answer {
            break;
        }
    }

    current_answer
}

/// GetDateTime: seconds since 1 January 1904 (the Mac's epoch), local
/// time taken as UTC.
pub fn get_date_time() -> u32 {
    let unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (unix + 2_082_844_800) as u32
}
