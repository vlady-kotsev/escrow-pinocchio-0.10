use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

pub struct Mint;

impl Mint {
    pub fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_len() != pinocchio_token::state::Mint::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

pub struct Token;
impl Token {
    pub fn check(account: &AccountView) -> Result<(), ProgramError> {
        if !account.owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account
            .data_len()
            .ne(&pinocchio_token::state::TokenAccount::LEN)
        {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

pub struct AssociatedToken;
impl AssociatedToken {
    pub fn check(
        account: &AccountView,
        authority: &AccountView,
        mint: &AccountView,
        token_program: &AccountView,
        bump: u8,
    ) -> Result<(), ProgramError> {
        Token::check(account)?;

        if derive_address(
            &[
                authority.address().as_array(),
                token_program.address().as_array(),
                mint.address().as_array(),
            ],
            Some(bump),
            pinocchio_associated_token_account::ID.as_array(),
        )
        .ne(account.address().as_array())
        {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }
    pub fn init(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
    ) -> ProgramResult {
        Create {
            funding_account: payer,
            account,
            wallet: owner,
            mint,
            system_program,
            token_program,
        }
        .invoke()
    }

    pub fn init_if_needed(
        account: &AccountView,
        mint: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        system_program: &AccountView,
        token_program: &AccountView,
        bump: u8,
    ) -> ProgramResult {
        match Self::check(account, owner, mint, token_program, bump) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, mint, payer, owner, system_program, token_program),
        }
    }
}

pub struct ProgramAccount;
impl ProgramAccount {
    pub fn check<T: Sized, const N: usize>(
        account: &AccountView,
        seeds: Option<&[&[u8]; N]>,
        bump: Option<u8>,
        space: usize,
    ) -> Result<(), ProgramError> {
        if !account.owned_by(&Address::new_from_array(crate::ID)) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_len().ne(&space) {
            return Err(ProgramError::InvalidAccountData);
        }

        if let Some(seeds) = seeds {
            let pda = derive_address(seeds, bump, &crate::ID);
            if pda.ne(account.address().as_array()) {
                return Err(ProgramError::InvalidSeeds);
            }
        }

        Ok(())
    }

    pub fn init<'a, T: Sized>(
        payer: &AccountView,
        account: &AccountView,
        seeds: &[Seed<'a>],
        space: usize,
    ) -> ProgramResult {
        let lamports = Rent::get()?.try_minimum_balance(space)?;
        let signer = [Signer::from(seeds)];
        // Create the account
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: space as u64,
            owner: &Address::new_from_array(crate::ID),
        }
        .invoke_signed(&signer)?;
        Ok(())
    }

    pub fn close(account: &AccountView, destination: &AccountView) -> ProgramResult {
        {
            let data = account.data_ptr();
            if account.data_len() > 0 {
                unsafe {
                    *data = 0xff;
                }
            }
        }
        destination.set_lamports(destination.lamports() + account.lamports());
        account.resize(1)?;
        account.close()
    }
}
