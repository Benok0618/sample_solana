use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, events::ThresholdExceeded, state::{Device, Reading}};

#[derive(Accounts)]
#[instruction(device_id: String)]
pub struct SubmitReading<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub device_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [DEVICE_SEED, device_id.as_bytes()],
        bump = device.bump,
    )]
    pub device: Account<'info, Device>,
    #[account(
        init,
        payer = payer,
        space = 8 + Reading::INIT_SPACE,
        seeds = [READING_SEED, device.key().as_ref(), &device.reading_count.to_le_bytes()],
        bump
    )]
    pub reading: Account<'info, Reading>,
    pub system_program: Program<'info, System>,
}

pub fn handle_submit_reading(
    ctx: Context<SubmitReading>,
    _device_id: String,
    value: i64,
    timestamp: i64,
) -> Result<()> {
    require!(ctx.accounts.device.active, ErrorCode::DeviceInactive);
    require_keys_eq!(
        ctx.accounts.device.authority,
        ctx.accounts.device_authority.key(),
        ErrorCode::UnauthorizedDevice,
    );

    let device = &mut ctx.accounts.device;
    let reading = &mut ctx.accounts.reading;

    reading.device = device.key();
    reading.value = value;
    reading.timestamp = timestamp;
    reading.index = device.reading_count;

    device.reading_count += 1;
    device.last_value = value;
    device.last_timestamp = timestamp;

    msg!("Reading #{} for {}: {}", reading.index, device.device_id, value);

    if value >= THRESHOLD_HIGH || value <= THRESHOLD_LOW {
        emit!(ThresholdExceeded {
            device: device.key(),
            device_id: device.device_id.clone(),
            value,
            timestamp,
        });
        msg!("Threshold exceeded for {}: {}", device.device_id, value);
    }

    Ok(())
}