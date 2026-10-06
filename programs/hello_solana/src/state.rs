use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct Device {
    pub authority: Pubkey,
    pub admin: Pubkey,
    #[max_len(32)]
    pub device_id: String,
    pub active: bool,
    pub reading_count: u64,
    pub last_value: i64,
    pub last_timestamp: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Reading {
    pub device: Pubkey,
    pub value: i64,
    pub timestamp: i64,
    pub index: u64,
}