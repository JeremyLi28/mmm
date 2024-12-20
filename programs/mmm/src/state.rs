use std::ops::Deref;

use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};
use mpl_bubblegum::accounts::TreeConfig;

use crate::constants::*;

pub const CURVE_KIND_LINEAR: u8 = 0;
pub const CURVE_KIND_EXP: u8 = 1;

pub const ALLOWLIST_KIND_EMPTY: u8 = 0;
pub const ALLOWLIST_KIND_FVCA: u8 = 1;
pub const ALLOWLIST_KIND_MINT: u8 = 2;
pub const ALLOWLIST_KIND_MCC: u8 = 3;
pub const ALLOWLIST_KIND_METADATA: u8 = 4;
pub const ALLOWLIST_KIND_GROUP: u8 = 5;
pub const ALLOWLIST_KIND_MPL_CORE_COLLECTION: u8 = 6;
// ANY nft will pass the allowlist check, please make sure to use cosigner to check NFT validity
pub const ALLOWLIST_KIND_ANY: u8 = u8::MAX;

#[derive(Default, Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Allowlist {
    pub kind: u8,
    pub value: Pubkey,
}

impl Allowlist {
    // kind == 0: empty
    // kind == 1: first verified creator address (FVCA)
    // kind == 2: single mint, useful for SFT
    // kind == 3: verified MCC
    // kind == 4: metadata
    // kind == 5: group extension
    // kind == 6: upgrade authority
    // kind == 7,8,... will be supported in the future
    // kind == 255: any
    pub fn valid(&self) -> bool {
        if self.kind > ALLOWLIST_KIND_MPL_CORE_COLLECTION && self.kind != ALLOWLIST_KIND_ANY {
            return false;
        }
        if self.kind != 0 && self.kind != ALLOWLIST_KIND_ANY {
            return self.value.ne(&Pubkey::default());
        }
        true
    }

    pub fn is_empty(&self) -> bool {
        self.kind == ALLOWLIST_KIND_EMPTY
    }
}

// seeds = [
//    POOL_PREFIX.as_bytes(),
//    owner.key().as_ref(),
//    pool.uuid.as_ref(),
// ]
#[account]
#[derive(Default)]
pub struct Pool {
    // mutable configurable
    pub spot_price: u64,
    pub curve_type: u8,
    pub curve_delta: u64,
    pub reinvest_fulfill_buy: bool,
    pub reinvest_fulfill_sell: bool,
    pub expiry: i64,
    pub lp_fee_bp: u16,
    pub referral: Pubkey,
    pub referral_bp: u16, // deprecated
    pub buyside_creator_royalty_bp: u16,

    // cosigner_annotation: it's set by the cosigner, could be the hash of the certain
    // free form of content, like collection_symbol, SFT name, and traits name
    // and etc. Needs to be carefully verified by the specific cosigner
    pub cosigner_annotation: [u8; 32],

    // mutable state data
    pub sellside_asset_amount: u64,
    pub lp_fee_earned: u64,

    // immutable
    pub owner: Pubkey,
    pub cosigner: Pubkey,
    pub uuid: Pubkey, // randomly generated keypair
    pub payment_mint: Pubkey,
    pub allowlists: [Allowlist; ALLOWLIST_MAX_LEN],
    pub buyside_payment_amount: u64,

    pub shared_escrow_account: Pubkey, // this points to the shared escrow account PDA (usually M2)
    pub shared_escrow_count: u64, // this means that how many times (count) the shared escrow account can be fulfilled, and it can be mutable
}

impl Pool {
    pub const LEN: usize = 8 +
        8 * 5 + // u64
        8 + // i64
        1 +  // u8
        2 * 2 +  // u16
        32 * 5 + // Pubkey
        2 + // bool
        32 + // [u8; 32]
        4 + (1 + 32) * ALLOWLIST_MAX_LEN + // Allowlist
        32 + // Pubkey
        8 + // u64
        352; // padding

    pub fn using_shared_escrow(&self) -> bool {
        self.shared_escrow_account != Pubkey::default()
    }
}

// seeds = [
//     SELL_STATE_PREFIX.as_bytes(),
//     pool.key().as_ref(),
//     asset_mint.key().as_ref(),
// ]
#[account]
#[derive(Default)]
pub struct SellState {
    // we are trying to normalize the info as much as possible
    // which means for indexing the SellState, we might need to
    // query the pool, but for convenience purpose, we added
    // cosigner_annotation here.
    //
    // we can add more fields for better indexing later.
    pub pool: Pubkey,
    pub pool_owner: Pubkey,
    pub asset_mint: Pubkey,
    pub asset_amount: u64,
    pub cosigner_annotation: [u8; 32],
}

impl SellState {
    pub const LEN: usize = 8 +
        8 + // u64
        32 * 3 + // Pubkey
        32 + // [u8; 32]
        200; // padding
}

// Wrapper structs to replace the Anchor program types until the Metaplex libs have
// better Anchor support.
pub struct BubblegumProgram;

impl Id for BubblegumProgram {
    fn id() -> Pubkey {
        mpl_bubblegum::ID
    }
}

#[derive(Clone)]
pub struct TreeConfigAnchor(pub TreeConfig);

impl AccountDeserialize for TreeConfigAnchor {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self(TreeConfig::from_bytes(buf)?))
    }
}

impl anchor_lang::Owner for TreeConfigAnchor {
    fn owner() -> Pubkey {
        // pub use spl_token::ID is used at the top of the file
        mpl_bubblegum::ID
    }
}

// No-op since we can't write data to a foreign program's account.
impl AccountSerialize for TreeConfigAnchor {}

impl Deref for TreeConfigAnchor {
    type Target = TreeConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
