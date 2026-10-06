use anchor_lang::prelude::*;

#[event]
pub struct ThresholdExceeded {
    pub device: Pubkey,
    pub device_id: String,
    pub value: i64,
    pub timestamp: i64,
}