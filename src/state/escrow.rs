use solana_address::Address;
use wincode::SchemaRead;

#[repr(C)]
#[derive(SchemaRead)]
pub struct Escrow<'a> {
    pub maker: &'a Address,
    pub mint_a: &'a Address,
    pub mint_b: &'a Address,
    pub receive: &'a u64,
    pub seed: &'a u64,
    pub bump: &'a u8,
}

#[repr(C)]
#[derive(SchemaRead)]
pub struct EscrowMut<'a> {
    pub maker: &'a mut Address,
    pub mint_a: &'a mut Address,
    pub mint_b: &'a mut Address,
    pub receive: &'a mut u64,
    pub seed: &'a mut u64,
    pub bump: &'a mut u8,
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
        *self.maker = *maker;
        *self.mint_a = *mint_a;
        *self.mint_b = *mint_b;
        *self.receive = *receive;
        *self.seed = *seed;
        *self.bump = *bump;
    }
}

impl<'a> Escrow<'a> {
    pub const LEN: usize = 3 * size_of::<Address>() + 2 * size_of::<u64>() + size_of::<u8>();
}
