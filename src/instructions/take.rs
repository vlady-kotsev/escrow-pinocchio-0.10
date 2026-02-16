use std::slice::from_raw_parts;

use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_token::instructions::{CloseAccount, Transfer};
use wincode::{SchemaRead, SchemaWrite, deserialize};

use crate::{
    constants::ESCROW_SEED,
    instructions::helpers::{AssociatedToken, Mint, ProgramAccount},
    state::Escrow,
};

pub struct TakeAccounts<'a> {
    pub taker: &'a AccountView,
    pub maker: &'a AccountView,
    pub escrow: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub maker_b_ata: &'a AccountView,
    pub taker_a_ata: &'a AccountView,
    pub taker_b_ata: &'a AccountView,
    pub vault: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for TakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            taker,
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_b_ata,
            taker_a_ata,
            taker_b_ata,
            vault,
            system_program,
            token_program,
            ..,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !taker.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if escrow.is_data_empty() {
            return Err(ProgramError::InvalidAccountData);
        }

        if !escrow.owned_by(&Address::new_from_array(crate::ID)) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(TakeAccounts {
            taker,
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_b_ata,
            taker_a_ata,
            taker_b_ata,
            vault,
            system_program,
            token_program,
        })
    }
}

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct TakeInstructionData {
    pub seed: u64,
    pub taker_a_ata_bump: u8,
    pub taker_b_ata_bump: u8,
    pub maker_b_ata_bump: u8,
    pub vault_ata_bump: u8,
}

impl<'a> TryFrom<&'a [u8]> for TakeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() + 4 * size_of::<u8>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let take_instruction_data = deserialize::<TakeInstructionData>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(take_instruction_data)
    }
}

pub struct Take<'a> {
    pub accounts: TakeAccounts<'a>,
    pub data: TakeInstructionData,
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for Take<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let data: TakeInstructionData = data.try_into()?;
        let accounts: TakeAccounts = accounts.try_into()?;

        Mint::check(accounts.mint_a)?;
        Mint::check(accounts.mint_b)?;

        AssociatedToken::check(
            accounts.taker_b_ata,
            accounts.taker,
            accounts.mint_b,
            accounts.token_program,
            data.taker_b_ata_bump,
        )?;

        AssociatedToken::check(
            accounts.vault,
            accounts.escrow,
            accounts.mint_a,
            accounts.token_program,
            data.vault_ata_bump,
        )?;

        Ok(Take { accounts, data })
    }
}

impl<'a> Take<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;
    pub fn process(&self) -> ProgramResult {
        AssociatedToken::init_if_needed(
            self.accounts.taker_a_ata,
            self.accounts.mint_a,
            self.accounts.taker,
            self.accounts.taker,
            self.accounts.system_program,
            self.accounts.token_program,
            self.data.taker_a_ata_bump,
        )?;

        AssociatedToken::init_if_needed(
            self.accounts.maker_b_ata,
            self.accounts.mint_b,
            self.accounts.taker,
            self.accounts.maker,
            self.accounts.system_program,
            self.accounts.token_program,
            self.data.maker_b_ata_bump,
        )?;

        let escrow_data = unsafe {
            from_raw_parts(
                self.accounts.escrow.data_ptr(),
                self.accounts.escrow.data_len(),
            )
        };
        let escrow =
            deserialize::<Escrow>(escrow_data).map_err(|_| ProgramError::InvalidInstructionData)?;

        let escrow_seeds: &[&[u8]; 3] = &[
            ESCROW_SEED,
            self.accounts.maker.address().as_array(),
            &self.data.seed.to_le_bytes(),
        ];
        ProgramAccount::check::<Escrow, 3>(
            self.accounts.escrow,
            Some(escrow_seeds),
            Some(*escrow.get_bump()),
            Escrow::LEN,
        )?;

        if escrow.get_mint_b().ne(self.accounts.mint_b.address()) {
            return Err(ProgramError::InvalidAccountData);
        }

        let vault_amount =
            pinocchio_token::state::TokenAccount::from_account_view(self.accounts.vault)?.amount();

        let seed_binding = self.data.seed.to_le_bytes();
        let bump_binding = &[*escrow.get_bump()];
        let seeds = &[
            Seed::from(ESCROW_SEED),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(bump_binding),
        ];
        let signers = &[Signer::from(seeds)];

        Transfer {
            from: self.accounts.vault,
            authority: self.accounts.escrow,
            to: self.accounts.taker_a_ata,
            amount: vault_amount,
        }
        .invoke_signed(signers)?;

        Transfer {
            from: self.accounts.taker_b_ata,
            to: self.accounts.maker_b_ata,
            authority: self.accounts.taker,
            amount: *escrow.get_receive(),
        }
        .invoke()?;

        CloseAccount {
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow,
        }
        .invoke_signed(signers)?;

        ProgramAccount::close(self.accounts.escrow, self.accounts.taker)
    }
}
