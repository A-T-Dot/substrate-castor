
//! Some configurable implementations as associated type for the substrate runtime.

use sr_primitives::traits::{Convert};
use super::Balance;

/// Handles converting a scalar to convert fee to energy point
/// 
pub struct FeeToEnergy;
impl Convert<Balance, Balance> for FeeToEnergy {
	fn convert(x: Balance) -> Balance {
    x.into()
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
