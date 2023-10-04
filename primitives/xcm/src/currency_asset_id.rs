use codec::{Decode, Encode, MaxEncodedLen, Compact};
use sp_std::marker::PhantomData;
use zenlink_protocol::GenerateLpAssetId;
pub type NewZenlinkAssetId = zenlink_protocol::AssetId;
use sp_runtime::traits::Convert;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
// use frame_support::traits::tokens::AssetId as AssetIdT;


/// Id used for identifying assets.
///
/// AssetId allocation:
/// [1; 2^32-1]     Custom user assets (permissionless)
/// [2^32; 2^64-1]  Statemine assets (simple map)
/// [2^64; 2^128-1] Ecosystem assets
/// 2^128-1         Relay chain token (KSM)
pub type PeaqAssetId = NewCurrencyId;

#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
struct PeaqInternalWrapId(pub PeaqAssetId);
const PARA_CHAIN_ID: u32 = 2000;
use sp_std::{
	convert::{Into, TryFrom},
};

// PeaqAssetId <> NewZenlinkAssetId

pub struct PeaqAssetIdZenlinkAssetIdConvertor<AssetId>(PhantomData<AssetId>);

impl<AssetId> Convert<AssetId, NewZenlinkAssetId> for PeaqAssetIdZenlinkAssetIdConvertor<AssetId>
where AssetId: Clone, u64: From<AssetId>
{
	fn convert(n: AssetId) -> NewZenlinkAssetId {
		let num: u64 = n.try_into().unwrap();
		let asset_type = if num == 0 {
			zenlink_protocol::NATIVE
		} else {
			zenlink_protocol::LOCAL
		};
		NewZenlinkAssetId {
			chain_id: PARA_CHAIN_ID,
			asset_type: asset_type,
			asset_index: num,
		}
	}
}

impl<AssetId> Convert<NewZenlinkAssetId, Option<AssetId>> for PeaqAssetIdZenlinkAssetIdConvertor<AssetId>
where AssetId: Clone + From<u64>
{
	// asset_id (
	fn convert(asset_id: NewZenlinkAssetId) -> Option<AssetId> {
		if asset_id.chain_id == PARA_CHAIN_ID {
			match asset_id.asset_type {
				zenlink_protocol::NATIVE => Some(0.into()),
				zenlink_protocol::LOCAL => Some((asset_id.asset_index).into()),
				_ => None,
			}
		} else {
			None
		}
	}
}

/*
 * impl PeaqInternalWrapId {
 *     pub fn is_native_token(&self) -> bool {
 *         self.0 == 0
 *     }
 *
 *     pub fn as_zenlink_asset_type(&self) -> u8 {
 *         if self.is_native_token() {
 *             zenlink_protocol::NATIVE
 *         } else {
 *             zenlink_protocol::LOCAL
 *         }
 *     }
 *
 *     pub fn as_zenlink_asset_index(&self) -> u64 {
 *         self.0 as u64
 *     }
 * }
 */
/*
 * impl PeaqAssetId {
 *     pub fn is_native_token(&self) -> bool {
 *         *self == 0
 *     }
 *
 *     pub fn as_zenlink_asset_type(&self) -> u8 {
 *         if self.is_native_token() {
 *             zenlink_protocol::NATIVE
 *         } else {
 *             zenlink_protocol::LOCAL
 *         }
 *     }
 *
 *     pub fn as_zenlink_asset_index(&self) -> u64 {
 *         *self as u64
 *     }
 * }
 */

/*
 * impl From<PeaqInternalWrapId> for NewZenlinkAssetId {
 *     fn from(ts: PeaqInternalWrapId) -> NewZenlinkAssetId {
 *         NewZenlinkAssetId {
 *             chain_id: PARA_CHAIN_ID,
 *             asset_type: ts.as_zenlink_asset_type(),
 *             asset_index: ts.as_zenlink_asset_index(),
 *         }
 *     }
 * }
 *
 * impl From<PeaqAssetId> for PeaqInternalWrapId {
 *     fn from(asset_id: PeaqAssetId) -> PeaqInternalWrapId {
 *         PeaqInternalWrapId(asset_id)
 *     }
 * }
 *
 * impl TryInto<PeaqAssetId> for PeaqInternalWrapId {
 *     type Error = ();
 *     fn try_into(self) -> Result<PeaqAssetId, Self::Error> {
 *         if self.is_native_token() {
 *             Err(())
 *         } else {
 *             Ok(self.0)
 *         }
 *     }
 * }
 *
 * pub fn try_convert(asset_id: NewZenlinkAssetId) -> Result<PeaqAssetId, ()> {
 *     PeaqInternalWrapId::try_from(asset_id)?.try_into()
 * }
 *
 * impl TryFrom<NewZenlinkAssetId> for PeaqInternalWrapId {
 *     type Error = ();
 *     // Convert from Zenlink AssetId to Peaq AssetId
 *     fn try_from(asset_id: NewZenlinkAssetId) -> Result<Self, Self::Error> {
 *         if asset_id.chain_id == PARA_CHAIN_ID	{
 *             Ok((asset_id.asset_index as u128).into())
 *         } else {
 *             Err(())
 *         }
 *     }
 * }
 */

#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum NewCurrencyId {
	/// All Polkadot based tokens (SS58-address-style), Relaychain- and Parachain-Tokens.
	Token(u64),
	/// Liquidity Pairs (Pairs of Tokens) within the PEAQ-Parachain.
	LPToken(u64, u64),
}

/*
 * impl AssetIdT for NewCurrencyId {
 * }
 */

impl NewCurrencyId {
	pub fn is_token(&self) -> bool {
		matches!(self, NewCurrencyId::Token(_))
	}

	pub fn is_lp_token(&self) -> bool {
		matches!(self, NewCurrencyId::LPToken(_, _))
	}

	pub fn is_native_token(&self) -> bool {
		if let NewCurrencyId::Token(symbol) = self {
			*symbol == 0 as u64
		} else {
			false
		}
	}

	// Internal method which simplifies conversions between Zenlink's asset_index
	fn type_index(&self) -> u64 {
		match self {
			NewCurrencyId::Token(_) => 0,
			NewCurrencyId::LPToken(_, _) => 1,
		}
	}
}

impl TryFrom<NewCurrencyId> for NewZenlinkAssetId {
	type Error = ();

	fn try_from(currency_id: NewCurrencyId) -> Result<Self, Self::Error> {
		match currency_id {
			NewCurrencyId::Token(symbol) => {
				let asset_type = if symbol == 0 {
					zenlink_protocol::NATIVE
				} else {
					zenlink_protocol::LOCAL
				};
				Ok(NewZenlinkAssetId {
					chain_id: PARA_CHAIN_ID,
					asset_type: asset_type,
					asset_index: symbol as u64,
				})
			},
			NewCurrencyId::LPToken(symbol0, symbol1) => Ok(NewZenlinkAssetId {
				chain_id: PARA_CHAIN_ID,
				asset_type: zenlink_protocol::LOCAL,
				asset_index: (currency_id.type_index() << 8) +
					((symbol0 as u64) << 16) +
					((symbol1 as u64) << 24),
			}),
			_ => Err(()),
		}
	}
}

impl TryFrom<NewZenlinkAssetId> for NewCurrencyId {
	type Error = ();

	fn try_from(asset_id: NewZenlinkAssetId) -> Result<Self, Self::Error> {
		if asset_id.chain_id == PARA_CHAIN_ID {
			let type_index = (asset_id.asset_index & 0x0000_0000_0000_ff00) >> 8 as u8;
			match type_index {
				0 => {
					let symbol = (asset_id.asset_index & 0x0000_0000_0000_00ff) as u64;
					Ok(NewCurrencyId::Token(symbol))
				},
				1 => {
					let symbol0 = ((asset_id.asset_index & 0x0000_0000_00ff_0000) >> 16) as u64;
					let symbol1 = ((asset_id.asset_index & 0x0000_0000_ff00_0000) >> 24) as u64;
					Ok(NewCurrencyId::LPToken(symbol0, symbol1))
				},
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}


impl Default for NewCurrencyId {
	fn default() -> Self {
		NewCurrencyId::Token(0 as u64)
	}
}

/// TODO: The local asset id should from 0 ~ 0x_ffff_ffff
/// This is the Peaq's default GenerateLpAssetId implementation.
pub struct NewPeaqZenlinkLpGenerate<T>(PhantomData<T>);

impl<T> GenerateLpAssetId<NewZenlinkAssetId> for NewPeaqZenlinkLpGenerate<T>
{
	fn generate_lp_asset_id(
		asset0: NewZenlinkAssetId,
		asset1: NewZenlinkAssetId,
	) -> Option<NewZenlinkAssetId> {
		let asset_id0: PeaqAssetId = asset0.try_into().ok()?;
		let asset_id1: PeaqAssetId = asset1.try_into().ok()?;

		match (asset_id0, asset_id1) {
			(NewCurrencyId::Token(symbol0), NewCurrencyId::Token(symbol1)) => {
				NewCurrencyId::LPToken(symbol0, symbol1).try_into().ok()
			},
			(_, _) => None,
		}
	}
}