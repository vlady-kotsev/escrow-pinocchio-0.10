use std::slice::from_raw_parts_mut;

use pinocchio::{AccountView, ProgramResult, cpi::Seed, error::ProgramError};
use pinocchio_system::ID as SYSTEM_PROGRAM_ID;
use pinocchio_token::instructions::Transfer;
use wincode::{SchemaRead, SchemaWrite, deserialize, serialize};

use crate::{
    constants::ESCROW_SEED,
    instructions::helpers::{AssociatedToken, Mint, ProgramAccount},
    state::Escrow,
};

pub struct MakeAccounts<'a> {
    pub maker: &'a AccountView,
    pub escrow: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub maker_ata: &'a AccountView,
    pub vault: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for MakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata,
            vault,
            system_program,
            token_program,
            ..,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !maker.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !escrow.is_data_empty() {
            return Err(ProgramError::InvalidAccountData);
        }

        if !escrow.owned_by(&SYSTEM_PROGRAM_ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(MakeAccounts {
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata,
            vault,
            system_program,
            token_program,
        })
    }
}

#[derive(SchemaRead, SchemaWrite)]
#[repr(C)]
pub struct MakeInstructionData {
    pub seed: u64,
    pub receive: u64,
    pub amount: u64,
    pub maker_ata_bump: u8,
    pub escrow_bump: u8,
}

impl<'a> TryFrom<&'a [u8]> for MakeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != 3 * size_of::<u64>() + 2 * size_of::<u8>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let make_instruction_data = deserialize::<MakeInstructionData>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        if make_instruction_data.amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(make_instruction_data)
    }
}

pub struct Make<'a> {
    pub accounts: MakeAccounts<'a>,
    pub data: MakeInstructionData,
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for Make<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts: MakeAccounts = accounts.try_into()?;
        let data: MakeInstructionData = data.try_into()?;

        Mint::check(accounts.mint_a)?;
        Mint::check(accounts.mint_b)?;
        AssociatedToken::check(
            accounts.maker_ata,
            accounts.maker,
            accounts.mint_a,
            accounts.token_program,
            data.maker_ata_bump,
        )?;

        Ok(Make { accounts, data })
    }
}

impl<'a> Make<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;
    pub fn process(&self) -> ProgramResult {
        let seed_binding = self.data.seed.to_le_bytes();
        let bump_binding = &[self.data.escrow_bump];
        let seeds = &[
            Seed::from(ESCROW_SEED),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(bump_binding),
        ];
        let space = Escrow::LEN;
        ProgramAccount::init::<Escrow>(self.accounts.maker, self.accounts.escrow, seeds, space)?;

        AssociatedToken::init(
            self.accounts.vault,
            self.accounts.mint_a,
            self.accounts.maker,
            self.accounts.escrow,
            self.accounts.system_program,
            self.accounts.token_program,
        )?;

        let escrow = Escrow::new(
            self.accounts.maker.address(),
            self.accounts.mint_a.address(),
            self.accounts.mint_b.address(),
            self.data.receive,
            self.data.seed,
            self.data.escrow_bump,
        );

        let escrow_account_data = unsafe {
            from_raw_parts_mut(
                self.accounts.escrow.data_ptr(),
                self.accounts.escrow.data_len(),
            )
        };
        let escrow_bytes =
            serialize::<Escrow>(&escrow).map_err(|_| ProgramError::InvalidInstructionData)?;
        escrow_account_data[..escrow_bytes.len()].copy_from_slice(&escrow_bytes);

        Transfer {
            from: self.accounts.maker_ata,
            authority: self.accounts.maker,
            to: self.accounts.vault,
            amount: self.data.amount,
        }
        .invoke()
    }
}
