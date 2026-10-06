use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, state::Device};

#[derive(Accounts)]
#[instruction(device_id: String)]
pub struct RegisterDevice<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: the wallet that will be authorized to submit readings for this device
    pub device_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Device::INIT_SPACE,
        seeds = [DEVICE_SEED, device_id.as_bytes()],
        bump
    )]
    pub device: Account<'info, Device>,
    pub system_program: Program<'info, System>,
}

pub fn handle_register_device(ctx: Context<RegisterDevice>, device_id: String) -> Result<()> {
    require!(device_id.len() <= MAX_DEVICE_ID_LEN, ErrorCode::DeviceIdTooLong);

    let device = &mut ctx.accounts.device;
    device.authority = ctx.accounts.device_authority.key();
    device.admin = ctx.accounts.admin.key();
    device.device_id = device_id;
    device.active = true;
    device.reading_count = 0;
    device.last_value = 0;
    device.last_timestamp = 0;
    device.bump = ctx.bumps.device;

    msg!("Device registered: {}", device.device_id);
    Ok(())
}