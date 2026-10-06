pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3sQrQZFU76P77zApJKTUhfDRy4SEZHutYUnbxMuqsYXN");

#[program]
pub mod hello_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        crate::instructions::initialize::handle_initialize(ctx)
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        crate::instructions::increment::handle_increment(ctx)
    }

    pub fn register_device(ctx: Context<RegisterDevice>, device_id: String) -> Result<()> {
        crate::instructions::register_device::handle_register_device(ctx, device_id)
    }

    pub fn submit_reading(
        ctx: Context<SubmitReading>,
        device_id: String,
        value: i64,
        timestamp: i64,
    ) -> Result<()> {
        crate::instructions::submit_reading::handle_submit_reading(ctx, device_id, value, timestamp)
    }

    pub fn set_device_active(
        ctx: Context<SetDeviceActive>,
        device_id: String,
        active: bool,
    ) -> Result<()> {
        crate::instructions::set_device_active::handle_set_device_active(ctx, device_id, active)
    }
}
