
//! Some configurable implementations as associated type for the substrate runtime.

use rstd::{convert::{TryInto}};
use sr_primitives::traits::{Convert};
use super::{Balance, BlockNumber};

/// Handles converting a scalar to convert fee to energy point
/// 
pub struct FeeToEnergy;
impl Convert<Balance, Balance> for FeeToEnergy {
	fn convert(x: Balance) -> Balance {
    x.into()
	}
}

/// Handles converting a scalar to convert energy point to fee
/// 
pub struct EnergyToFee;
impl Convert<Balance, Balance> for EnergyToFee {
	fn convert(x: Balance) -> Balance {
    x.into()
	}
}

pub struct EnergyToLocking;
impl Convert<Balance, BlockNumber> for EnergyToLocking {
	fn convert(x: Balance) -> BlockNumber {
    x.try_into().unwrap()
	}
}

/// Handles converting a scalar to convert charging currency to energy point
/// 
pub struct ChargingToEnergy;
impl Convert<Balance, Balance> for ChargingToEnergy {
	fn convert(x: Balance) -> Balance {
		// 1: 1000
    Balance::from(x).saturating_mul(10)
	}
}

/// Handles converting a scalar to convert balance
/// 
pub struct ConvertBalance;
impl Convert<Balance, Balance> for ConvertBalance {
	fn convert(x: Balance) -> Balance {
    x.into()
	}
}