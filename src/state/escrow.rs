#![allow(unused)]
use pinocchio::Address;
use wincode::{SchemaRead, SchemaWrite};

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct Escrow {
    maker: [u8; 32],
    mint_a: [u8; 32],
    mint_b: [u8; 32],
    receive: u64,
    seed: u64,
    bump: u8,
}

impl Escrow {
    pub const LEN: usize = size_of::<Escrow>();

    pub fn new(
        maker: &Address,
        mint_a: &Address,
        mint_b: &Address,
        receive: u64,
        seed: u64,
        bump: u8,
    ) -> Escrow {
        Escrow {
            maker: maker.to_bytes(),
            mint_a: mint_a.to_bytes(),
            mint_b: mint_b.to_bytes(),
            receive,
            seed,
            bump,
        }
    }

    pub fn get_maker(&self) -> Address {
        unsafe { std::mem::transmute::<[u8; 32], Address>(self.maker) }
    }

    pub fn get_mint_a(&self) -> Address {
        unsafe { std::mem::transmute::<[u8; 32], Address>(self.mint_a) }
    }

    pub fn get_mint_b(&self) -> Address {
        unsafe { std::mem::transmute::<[u8; 32], Address>(self.mint_b) }
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn get_receive(&self) -> u64 {
        self.receive
    }

    pub fn get_bump(&self) -> u8 {
        self.bump
    }
}
