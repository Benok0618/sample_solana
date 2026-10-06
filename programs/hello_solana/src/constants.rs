use anchor_lang::prelude::*;

#[constant]
pub const COUNTER_SEED: &[u8] = b"counter";

#[constant]
pub const HELLO_WORLD_LAMPORTS: u64 = 1;

#[constant]
pub const MAX_COUNT: u64 = 10;

#[constant]
pub const DEVICE_SEED: &[u8] = b"device";

#[constant]
pub const READING_SEED: &[u8] = b"reading";

pub const MAX_DEVICE_ID_LEN: usize = 32;

#[constant]
pub const THRESHOLD_HIGH: i64 = 1000;

#[constant]
pub const THRESHOLD_LOW: i64 = -1000;
