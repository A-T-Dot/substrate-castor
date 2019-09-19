/// tests for this module
#[!cfg(test)]

use runtime_io::with_externalities;
use primitives::{H256, Blake2Hasher};
use support::{impl_outer_origin, assert_ok, parameter_types};
use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
use sr_primitives::weights::Weight;
use sr_primitives::Perbill;

impl_outer_origin! {
  pub enum Origin for Test {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const MaximumBlockWeight: Weight = 1024;
  pub const MaximumBlockLength: u32 = 2 * 1024;
  pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl system::Trait for Test {
  type Origin = Origin;
  type Call = ();
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = u64;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type WeightMultiplierUpdate = ();
  type Event = ();
  type BlockHashCount = BlockHashCount;
  type MaximumBlockWeight = MaximumBlockWeight;
  type MaximumBlockLength = MaximumBlockLength;
  type AvailableBlockRatio = AvailableBlockRatio;
  type Version = ();
}
pub type Balance = u128;

impl balances::Trait for Test {
  type Balance = Balance;
  /// What to do if an account's free balance gets zeroed.
  type OnFreeBalanceZero = ();
  /// What to do if a new account is created.
  type OnNewAccount = ();
  /// The ubiquitous event type.
  type Event = ();
  type TransactionPayment = ();
  type DustRemoval = ();
  type TransferPayment = ();
  type ExistentialDeposit = ();
  type TransferFee = ();
  type CreationFee = ();
  type TransactionBaseFee = ();
  type TransactionByteFee = ();
  type WeightToFee = ();
}

impl Trait for Test {
  type Event = ();
  type GeId = u64;
  type Currency = balances::Module<Test>;
}
type Tcx = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
  system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

#[test]
fn it_creates_ge_correctly() {
  with_externalities(&mut new_test_ext(), || {

    assert_ok!(Tcx::create(Origin::signed(1)));
    assert_eq!(Tcx::governance_entities_count(), 1);
    assert_ok!(Tcx::stake(Origin::signed(1), 1, 100000000));
  });
}
