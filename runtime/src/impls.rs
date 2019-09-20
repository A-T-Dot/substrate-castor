
//! Some configurable implementations as associated type for the substrate runtime.

use sr_primitives::traits::{Convert};
use super::Balance;

/// Handles converting a scalar to convert fee to energy point
/// node's balance type.
/// 
pub struct FeeToEnergy;
impl Convert<Balance, Balance> for FeeToEnergy {
	fn convert(x: Balance) -> Balance {
    x.into()
	}
}
