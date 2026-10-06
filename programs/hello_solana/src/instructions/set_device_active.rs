use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, state::Device};

#[derive(Accounts)]
#[instruction(device_id: String)]
pub struct SetDeviceActive<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [DEVICE_SEED, device_id.as_bytes()],
        bump = device.bump,
    )]
    pub device: Account<'info, Device>,
}

pub fn handle_set_device_active(
    ctx: Context<SetDeviceActive>,
    _device_id: String,
    active: bool,
) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.device.admin,
        ctx.accounts.admin.key(),
        ErrorCode::UnauthorizedAdmin,
    );

    ctx.accounts.device.active = active;
    msg!("Device {} active = {}", ctx.accounts.device.device_id, active);
    Ok(())
}