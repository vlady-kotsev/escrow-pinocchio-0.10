#![allow(unused)]
use pinocchio::Address;
use wincode::{SchemaRead, SchemaWrite};

#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct Escrow<'a> {
    maker: &'a [u8; 32],
    mint_a: &'a [u8; 32],
    mint_b: &'a [u8; 32],
    receive: &'a u64,
    seed: &'a u64,
    bump: &'a u8,
}

#[repr(C)]
#[derive(SchemaRead)]
pub struct EscrowMut<'a> {
    maker: &'a mut [u8; 32],
    mint_a: &'a mut [u8; 32],
    mint_b: &'a mut [u8; 32],
    receive: &'a mut u64,
    seed: &'a mut u64,
    bump: &'a mut u8,
}

impl<'a> EscrowMut<'a> {
    pub fn set_inner(
        &mut self,
        maker: &'a Address,
        mint_a: &'a Address,
        mint_b: &'a Address,
        receive: &'a u64,
        seed: &'a u64,
        bump: &'a u8,
    ) {
        *self.maker = *maker.as_array();
        *self.mint_a = *mint_a.as_array();
        *self.mint_b = *mint_b.as_array();
        *self.receive = *receive;
        *self.seed = *seed;
        *self.bump = *bump;
    }
}

impl<'a> Escrow<'a> {
    pub const LEN: usize = 3 * size_of::<[u8; 32]>() + 2 * size_of::<u64>() + size_of::<u8>();

    pub fn get_maker(&self) -> &Address {
        unsafe { std::mem::transmute::<&[u8; 32], &Address>(self.maker) }
    }

    pub fn get_mint_a(&self) -> &Address {
        unsafe { std::mem::transmute::<&[u8; 32], &Address>(self.mint_a) }
    }

    pub fn get_mint_b(&self) -> &Address {
        unsafe { std::mem::transmute::<&[u8; 32], &Address>(self.mint_b) }
    }

    pub fn get_seed(&self) -> &u64 {
        self.seed
    }

    pub fn get_receive(&self) -> &u64 {
        self.receive
    }

    pub fn get_bump(&self) -> &u8 {
        self.bump
    }
}
