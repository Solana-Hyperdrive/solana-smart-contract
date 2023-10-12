use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::states::VaultState;

pub const SEED_MAIN_STATE: &[u8] = b"main";
pub const SEED_VAULT_STATE: &[u8] = b"vault";
pub const MAX_FEE_REATE: u128 = 1000_000;

pub(crate) fn transfer_tokens<'info>(
    authority: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let transfer_accounts = Transfer {
        to,
        from,
        authority,
    };

    token::transfer(CpiContext::new(token_program, transfer_accounts), amount)?;
    Ok(())
}

pub(crate) fn transfer_tokens_from_vault<'info>(
    vault: &mut Account<'info, VaultState>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
    bump: u8,
) -> Result<()> {
    let transfer_accounts = Transfer {
        to,
        from,
        authority: vault.to_account_info(),
    };

    token::transfer(
        CpiContext::new_with_signer(
            token_program,
            transfer_accounts,
            &[&[
                SEED_VAULT_STATE,
                vault.sender.as_ref(),
                vault.id.to_le_bytes().as_ref(),
                &[bump],
            ]],
        ),
        amount,
    )?;
    Ok(())
}
