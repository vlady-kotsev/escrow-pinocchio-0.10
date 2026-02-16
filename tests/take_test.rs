#[cfg(test)]
mod take_test {
    use escrow::{ESCROW_SEED, ID as PROGRAM_ID};
    use mollusk_svm::{Mollusk, program::keyed_account_for_system_program, result::Check};
    use mollusk_svm_programs_token::{
        associated_token::create_account_for_associated_token_account,
        token::create_account_for_mint,
    };
    use pinocchio::{Address, error::ProgramError};
    use pinocchio_system::ID as SYSTEM_PROGRAM_ID;
    use solana_account::Account;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_program_option::COption;
    use solana_rent::Rent;
    use spl_token_interface::state::{Account as TokenAccount, AccountState, Mint};

    const PROGRAM_ADDRESS: Address = Address::new_from_array(PROGRAM_ID);
    const SEED: u64 = 1;

    fn setup() -> (Mollusk, Vec<(Address, Account)>, Instruction) {
        let mut mollusk = Mollusk::new_debuggable(&PROGRAM_ADDRESS, "target/deploy/escrow", true);
        mollusk_svm_programs_token::token::add_program(&mut mollusk);
        mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);

        let (system_program_address, system_program_account) = keyed_account_for_system_program();
        let (token_program_address, token_program_account) =
            mollusk_svm_programs_token::token::keyed_account();
        let (associated_token_program_address, associated_token_program_account) =
            mollusk_svm_programs_token::associated_token::keyed_account();

        let maker_address = Address::new_unique();
        let maker_account = Account::new(100_000_000, 0, &SYSTEM_PROGRAM_ID);

        let taker_address = Address::new_unique();
        let taker_account = Account::new(100_000_000, 0, &SYSTEM_PROGRAM_ID);

        let mint_a_address = Address::new_unique();
        let mint_a_data = Mint {
            mint_authority: COption::Some(Address::new_unique()),
            supply: 10_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mint_a_account = create_account_for_mint(mint_a_data);

        let mint_b_address = Address::new_unique();
        let mint_b_data = Mint {
            mint_authority: COption::Some(Address::new_unique()),
            supply: 10_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mint_b_account = create_account_for_mint(mint_b_data);

        let (escrow_address, escrow_bump) = Address::find_program_address(
            &[
                ESCROW_SEED,
                maker_address.as_ref(),
                SEED.to_le_bytes().as_ref(),
            ],
            &PROGRAM_ADDRESS,
        );

        let expected_escrow_data = [
            maker_address.as_ref(),
            mint_a_address.as_ref(),
            mint_b_address.as_ref(),
            &1000u64.to_le_bytes(),
            &SEED.to_le_bytes(),
            &[escrow_bump],
        ]
        .concat();

        let escrow_len = expected_escrow_data.len();
        let escrow_lamports = Rent::default()
            .try_minimum_balance(escrow_len)
            .expect("Can't calculate rent");

        let mut escrow_account = Account::new(escrow_lamports, escrow_len, &PROGRAM_ADDRESS);
        escrow_account.data = expected_escrow_data;

        let (maker_b_ata_address, maker_b_ata_bump) = Address::find_program_address(
            &[
                maker_address.as_ref(),
                token_program_address.as_ref(),
                mint_b_address.as_ref(),
            ],
            &associated_token_program_address,
        );

        let (taker_a_ata_address, taker_a_ata_bump) = Address::find_program_address(
            &[
                taker_address.as_ref(),
                token_program_address.as_ref(),
                mint_a_address.as_ref(),
            ],
            &associated_token_program_address,
        );

        let taker_b_data = TokenAccount {
            mint: mint_b_address,
            owner: taker_address,
            amount: 1_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };
        let (taker_b_ata_address, taker_b_ata_account) =
            create_account_for_associated_token_account(taker_b_data);

        let vault_data = TokenAccount {
            mint: mint_a_address,
            owner: escrow_address,
            amount: 1_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };
        let (vault_address, vault_account) =
            create_account_for_associated_token_account(vault_data);

        let accounts = vec![
            (taker_address, taker_account),
            (maker_address, maker_account),
            (escrow_address, escrow_account),
            (mint_a_address, mint_a_account),
            (mint_b_address, mint_b_account),
            (maker_b_ata_address, Account::default()),
            (taker_a_ata_address, Account::default()),
            (taker_b_ata_address, taker_b_ata_account),
            (vault_address, vault_account),
            (system_program_address, system_program_account),
            (token_program_address, token_program_account),
            (
                associated_token_program_address,
                associated_token_program_account,
            ),
        ];

        let (_, taker_b_ata_bump) = Address::find_program_address(
            &[
                taker_address.as_ref(),
                token_program_address.as_ref(),
                mint_b_address.as_ref(),
            ],
            &associated_token_program_address,
        );

        let (_, vault_ata_bump) = Address::find_program_address(
            &[
                escrow_address.as_ref(),
                token_program_address.as_ref(),
                mint_a_address.as_ref(),
            ],
            &associated_token_program_address,
        );

        let discriminator = &[1u8];

        let instruction_data = [
            discriminator.as_ref(),
            &SEED.to_le_bytes(),
            &[taker_a_ata_bump],
            &[taker_b_ata_bump],
            &[maker_b_ata_bump],
            &[vault_ata_bump],
        ]
        .concat();

        let instruction = Instruction::new_with_bytes(
            PROGRAM_ADDRESS,
            &instruction_data,
            vec![
                AccountMeta::new(taker_address, true),
                AccountMeta::new(maker_address, false),
                AccountMeta::new(escrow_address, false),
                AccountMeta::new_readonly(mint_a_address, false),
                AccountMeta::new_readonly(mint_b_address, false),
                AccountMeta::new(maker_b_ata_address, false),
                AccountMeta::new(taker_a_ata_address, false),
                AccountMeta::new(taker_b_ata_address, false),
                AccountMeta::new(vault_address, false),
                AccountMeta::new_readonly(system_program_address, false),
                AccountMeta::new_readonly(token_program_address, false),
                AccountMeta::new_readonly(associated_token_program_address, false),
            ],
        );

        (
            mollusk,
            accounts,
            instruction,
            //vec![maker_a_ata_bump, escrow_bump],
        )
    }

    #[test]
    fn take_works_correctly() {
        let (mollusk, accounts, instruction) = setup();

        // taker_a_ata (accounts[6]): created, receives 1_000_000 mint_a from vault
        let expected_taker_a_ata_data = create_account_for_associated_token_account(TokenAccount {
            mint: accounts[3].0,
            owner: accounts[0].0,
            amount: 1_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        })
        .1
        .data;

        // maker_b_ata (accounts[5]): created, receives 1000 mint_b from taker
        let expected_maker_b_ata_data = create_account_for_associated_token_account(TokenAccount {
            mint: accounts[4].0,
            owner: accounts[1].0,
            amount: 1000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        })
        .1
        .data;

        // taker_b_ata (accounts[7]): 1_000_000 - 1000 = 999_000 mint_b
        let expected_taker_b_ata_data = create_account_for_associated_token_account(TokenAccount {
            mint: accounts[4].0,
            owner: accounts[0].0,
            amount: 999_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        })
        .1
        .data;

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[
                Check::success(),
                Check::account(&accounts[5].0)
                    .data(&expected_maker_b_ata_data)
                    .build(),
                Check::account(&accounts[6].0)
                    .data(&expected_taker_a_ata_data)
                    .build(),
                Check::account(&accounts[7].0)
                    .data(&expected_taker_b_ata_data)
                    .build(),
                Check::account(&accounts[8].0).lamports(0).data(&[]).build(),
            ],
        );
    }

    #[test]
    fn take_fails_missing_signer() {
        let (mollusk, accounts, instruction) = setup();

        let mut metas = instruction.accounts.clone();
        metas[0].is_signer = false;
        let instruction = Instruction::new_with_bytes(PROGRAM_ADDRESS, &instruction.data, metas);

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::MissingRequiredSignature)],
        );
    }

    #[test]
    fn take_fails_escrow_empty() {
        let (mollusk, mut accounts, instruction) = setup();

        accounts[2].1 = Account::default();

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::InvalidAccountData)],
        );
    }

    #[test]
    fn take_fails_escrow_wrong_owner() {
        let (mollusk, mut accounts, instruction) = setup();

        accounts[2].1.owner = Address::new_unique();

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::InvalidAccountOwner)],
        );
    }

    #[test]
    fn take_fails_wrong_mint_b() {
        let (mollusk, mut accounts, instruction) = setup();

        // Overwrite mint_b inside escrow data (bytes 64..96) with a fake address
        let fake_mint_b = Address::new_unique();
        accounts[2].1.data[64..96].copy_from_slice(fake_mint_b.as_ref());

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::InvalidAccountData)],
        );
    }

    #[test]
    fn take_fails_invalid_instruction_data() {
        let (mollusk, accounts, instruction) = setup();

        let instruction =
            Instruction::new_with_bytes(PROGRAM_ADDRESS, &[1u8; 3], instruction.accounts);

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::InvalidInstructionData)],
        );
    }

    #[test]
    fn take_fails_not_enough_accounts() {
        let (mollusk, accounts, instruction) = setup();

        let metas = instruction.accounts[..5].to_vec();
        let instruction = Instruction::new_with_bytes(PROGRAM_ADDRESS, &instruction.data, metas);

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(ProgramError::NotEnoughAccountKeys)],
        );
    }
}
