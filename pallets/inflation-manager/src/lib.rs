#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub use pallet::*;

pub mod types;
pub use types::{
	BalanceOf, InflationConfiguration as InflationConfigurationT,
	InflationParameters as InflationParametersT,
};

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, IsType},
};
use frame_system::WeightInfo;
use sp_runtime::{traits::BlockNumberProvider, Perbill};
pub const BLOCKS_PER_YEAR: peaq_primitives_xcm::BlockNumber = 365 * 24 * 60 * 60 / 12_u32;
use peaq_primitives_xcm::Balance;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency trait.
		type Currency: Currency<Self::AccountId, Balance = Balance>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// Bounds for BoundedVec across this pallet's storage
		#[pallet::constant]
		type BoundedDataLen: Get<u32>;
	}

	/// Inflation kicks off with these parameters
	#[pallet::storage]
	#[pallet::getter(fn base_inflation_parameters)]
	pub type InflationConfiguration<T: Config> =
		StorageValue<_, InflationConfigurationT, ValueQuery>;

	/// inflation parameters, calculated each year, based off of the base inflation parameters
	/// provided at genesis
	#[pallet::storage]
	#[pallet::getter(fn effective_inflation_parameters)]
	pub type YearlyInflationParameters<T: Config> =
		StorageValue<_, InflationParametersT, ValueQuery>;

	/// Info for how many years have passed, starting and ending at which block.
	#[pallet::storage]
	#[pallet::getter(fn current_year)]
	pub type CurrentYear<T: Config> = StorageValue<_, u128, ValueQuery>;

	/// Flag indicating whether on the first possible opportunity, recalculation of the inflation
	/// parameters should be done.
	/// New inflation parameters kick in from the next block after the recalculation block.
	#[pallet::storage]
	#[pallet::getter(fn recalculation_at)]
	pub type RecalculationAt<T: Config> = StorageValue<_, T::BlockNumber, OptionQuery>;

	/// The current rewards per block
	#[pallet::storage]
	#[pallet::getter(fn block_rewards)]
	pub type BlockRewards<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		// New fiscal year triggered with updated inflation, disinflation rate and inflationary
		// tokens to mint per block
		InflationParametersUpdated {
			inflation_parameters: InflationParametersT,
			block_rewards: Balance,
			current_year: u128,
		},
		InflationConfigurationSet {
			inflation_configuration: InflationConfigurationT,
		},
		BlockRewardsUpdated {
			block_rewards: Balance,
		},
	}

	/// Error for evm accounts module.
	#[pallet::error]
	pub enum Error<T> {
		FiscalYearUninitialized,
	}

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		pub inflation_configuration: InflationConfigurationT,
	}

	#[cfg(feature = "std")]
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			// install base inflation parameters
			InflationConfiguration::<T>::put(self.inflation_configuration.clone());
			YearlyInflationParameters::<T>::put(
				self.inflation_configuration.base_inflation_parameters.clone(),
			);

			// set current inflationary year
			CurrentYear::<T>::put(1);

			// set the flag to calculate inflation parameters after a year(in blocks)
			let racalculation_target_block = frame_system::Pallet::<T>::current_block_number() +
				T::BlockNumber::from(BLOCKS_PER_YEAR);

			// Update recalculation flag
			RecalculationAt::<T>::put(racalculation_target_block);

			let block_rewards = Pallet::<T>::rewards_per_block(
				&self.inflation_configuration.base_inflation_parameters,
			);

			BlockRewards::<T>::put(block_rewards);

			Pallet::<T>::deposit_event(Event::InflationConfigurationSet {
				inflation_configuration: self.inflation_configuration.clone(),
			});
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(now: T::BlockNumber) {
			// flag to check weather or not we're at end of a year and must recalculate block rewards
			let mut logic_switch: bool = false;

			let current_year = CurrentYear::<T>::get();
			let inflation_config = InflationConfiguration::<T>::get();
			let mut inflation_parameters = YearlyInflationParameters::<T>::get();
			let mut block_rewards = BlockRewards::<T>::get();

			// if we have reached the stagnation year, kill the recalculation flag
			// and set inflation parameters to stagnation values
			if current_year == inflation_config.inflation_stagnation_year {

				inflation_parameters = InflationParametersT {
					effective_inflation_rate: inflation_config.inflation_stagnation_rate,
					effective_disinflation_rate: Perbill::one(),
				};

				// kill recalculation flag
				RecalculationAt::<T>::kill();

				// put stagnation inflation parameters into storage
				YearlyInflationParameters::<T>::put(inflation_parameters.clone());

				logic_switch = true;
			}

			// check if we need to recalculate inflation parameters for a new year
			// update inflation parameters if we havent reached the stagnation year
			if let Some(target_block) = RecalculationAt::<T>::get() {
				if now >= target_block && current_year < inflation_config.inflation_stagnation_year
				{
					// update inflation parameters
					inflation_parameters =
						Self::update_inflation_parameters(&inflation_parameters.clone());
					YearlyInflationParameters::<T>::put(inflation_parameters.clone());

					// set the flag to calculate inflation parameters after a year(in blocks)
					let target_block = now + T::BlockNumber::from(BLOCKS_PER_YEAR);
					RecalculationAt::<T>::put(target_block);

					logic_switch = true;
				}
			}

			if logic_switch {
				// calculate new block rewards	
				block_rewards = Self::rewards_per_block(&inflation_parameters);

				// put new block rewards into storage
				BlockRewards::<T>::put(block_rewards);

				// log this change
				Self::deposit_event(Event::BlockRewardsUpdated {
					block_rewards,
				});
			}

			// update current year in any case
			CurrentYear::<T>::put(current_year + 1);

			// Event
			Self::deposit_event(Event::InflationParametersUpdated {
				current_year,
				inflation_parameters: inflation_parameters.clone(),
				block_rewards,
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {
		// calculate inflationary tokens per block
		fn rewards_per_block(inflation_parameters: &InflationParametersT) -> Balance {
			let total_issuance = T::Currency::total_issuance();
			let rewards_total = inflation_parameters.effective_inflation_rate * total_issuance;
			// TODO Verify this convesion
			rewards_total / Balance::from(BLOCKS_PER_YEAR)
		}

		// We do not expect this to underflow/overflow
		fn update_inflation_parameters(
			inflation_parameters: &InflationParametersT,
		) -> InflationParametersT {
			let base_inflation_parameters =
				InflationConfiguration::<T>::get().base_inflation_parameters;
			// Calculate effective disinflation rate as
			// effective_disinflation_rate(n) =
			// effective_disinflation_rate(0) * effective_disinflation_rate(n-1)
			let effective_disinflation_rate = inflation_parameters.effective_disinflation_rate *
				base_inflation_parameters.effective_disinflation_rate;

			// Calculate effective inflation rate as
			// effective_inflation_rate(n) =
			// effective_inflation_rate(n-1) * effective_disinflation_rate(n)
			let effective_inflation_rate =
				inflation_parameters.effective_inflation_rate * effective_disinflation_rate;

			InflationParametersT { effective_inflation_rate, effective_disinflation_rate }
		}
	}
}
