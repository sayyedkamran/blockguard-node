// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org

//! Unit testing

/*
 * use std::{convert::TryInto, iter};
 *
 * use crate::{
 *     mock::MockRewardConfig,
 *     reward_config_calc::{DefaultRewardCalculator, RewardRateConfigTrait},
 *     reward_rate::RewardRateInfo,
 * };
 * use frame_support::{
 *     assert_noop, assert_ok, storage::bounded_btree_map::BoundedBTreeMap,
 *     traits::EstimateNextSessionRotation, BoundedVec,
 * };
 * use frame_system::RawOrigin;
 * use pallet_balances::{BalanceLock, Error as BalancesError, Reasons};
 * use pallet_session::{SessionManager, ShouldEndSession};
 * use sp_runtime::{traits::Zero, Perbill, Permill, Perquintill, SaturatedConversion};
 *
 * use crate::{
 *     mock::{
 *         almost_equal, events, last_event, roll_to, AccountId, Balance, Balances, BlockNumber,
 *         ExtBuilder, RuntimeEvent as MetaEvent, RuntimeOrigin, Session, StakePallet, System,
 * Test,     },
 *     reward_config_calc::CollatorDelegatorBlockRewardCalculator,
 *     set::OrderedSet,
 *     types::{
 *         BalanceOf, Candidate, CandidateStatus, DelegationCounter, Delegator, Reward,
 * RoundInfo,     },
 *     Config, STAKING_ID,
 * };
 */

use crate::{
	mock::{
		roll_to, Balances, BlockNumber, ExtBuilder, PCall, Precompiles, PrecompilesValue,
		RuntimeOrigin, StakePallet, Test,
	},
	Address, BalanceOf,
};
use frame_support::{
	assert_noop, assert_ok,
	storage::bounded_btree_map::BoundedBTreeMap,
	traits::{EstimateNextSessionRotation, LockIdentifier},
	BoundedVec,
};
use pallet_balances::{BalanceLock, Reasons};
use parachain_staking::types::TotalStake;
use precompile_utils::testing::{MockPeaqAccount, PrecompileTesterExt};

const STAKING_ID: LockIdentifier = *b"kiltpstk";

fn precompiles() -> Precompiles<Test> {
	PrecompilesValue::get()
}

#[test]
fn unlock_unstaked() {
	// same_unstaked_as_restaked
	// block 1: stake & unstake for 100
	// block 2: stake & unstake for 100
	// should remove first entry in unstaking BoundedBTreeMap when staking in block
	// 2 should still have 100 locked until unlocking
	ExtBuilder::default()
		.with_balances(vec![(MockPeaqAccount::Alice, 10), (MockPeaqAccount::Bob, 100)])
		.with_collators(vec![(MockPeaqAccount::Alice, 10)])
		.with_delegators(vec![(MockPeaqAccount::Bob, MockPeaqAccount::Alice, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::revoke_delegation(
				RuntimeOrigin::signed(MockPeaqAccount::Bob),
				MockPeaqAccount::Alice
			));
			let mut unstaking: BoundedBTreeMap<
				BlockNumber,
				BalanceOf<Test>,
				<Test as parachain_staking::Config>::MaxUnstakeRequests,
			> = BoundedBTreeMap::new();
			assert_ok!(unstaking.try_insert(3, 100));
			let lock = BalanceLock { id: STAKING_ID, amount: 100, reasons: Reasons::All };
			assert_eq!(StakePallet::unstaking(MockPeaqAccount::Bob), unstaking);
			assert_eq!(Balances::locks(MockPeaqAccount::Bob), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(
				RuntimeOrigin::signed(MockPeaqAccount::Bob),
				MockPeaqAccount::Bob
			));
			assert_eq!(StakePallet::unstaking(MockPeaqAccount::Bob), unstaking);
			assert_eq!(Balances::locks(MockPeaqAccount::Bob), vec![lock.clone()]);

			// join delegators and revoke again --> consume unstaking at block 3
			roll_to(2, vec![]);
			precompiles()
				.prepare_test(
					MockPeaqAccount::Bob,
					MockPeaqAccount::EVMu1Account,
					PCall::join_delegators {
						collator: Address(MockPeaqAccount::Alice.into()),
						amount: 100.into(),
					},
				)
				.expect_no_logs()
				.execute_returns(());

			assert_ok!(StakePallet::revoke_delegation(
				RuntimeOrigin::signed(MockPeaqAccount::Bob),
				MockPeaqAccount::Alice
			));
			unstaking.remove(&3);
			assert_ok!(unstaking.try_insert(4, 100));
			assert_eq!(StakePallet::unstaking(MockPeaqAccount::Bob), unstaking);
			assert_eq!(Balances::locks(MockPeaqAccount::Bob), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(
				RuntimeOrigin::signed(MockPeaqAccount::Bob),
				MockPeaqAccount::Bob
			));
			assert_eq!(StakePallet::unstaking(MockPeaqAccount::Bob), unstaking);
			assert_eq!(Balances::locks(MockPeaqAccount::Bob), vec![lock.clone()]);

			// should reduce unlocking but not unlock anything
			roll_to(3, vec![]);
			assert_eq!(StakePallet::unstaking(MockPeaqAccount::Bob), unstaking);
			assert_eq!(Balances::locks(MockPeaqAccount::Bob), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(
				RuntimeOrigin::signed(MockPeaqAccount::Bob),
				MockPeaqAccount::Bob
			));
			assert_eq!(StakePallet::unstaking(MockPeaqAccount::Bob), unstaking);
			assert_eq!(Balances::locks(MockPeaqAccount::Bob), vec![lock.clone()]);

			roll_to(4, vec![]);
			unstaking.remove(&4);
			assert_eq!(Balances::locks(MockPeaqAccount::Bob), vec![lock]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(
				RuntimeOrigin::signed(MockPeaqAccount::Bob),
				MockPeaqAccount::Bob
			));
			assert_eq!(StakePallet::unstaking(MockPeaqAccount::Bob), unstaking);
			assert_eq!(Balances::locks(MockPeaqAccount::Bob), vec![]);
		});
}

#[test]
fn should_update_total_stake() {
	ExtBuilder::default()
		.with_balances(vec![
			(MockPeaqAccount::Alice, 100),
			(MockPeaqAccount::Bob, 100),
			(MockPeaqAccount::Charlie, 100),
			(MockPeaqAccount::David, 500),
			(MockPeaqAccount::ParentAccount, 100),
		])
		.with_collators(vec![(MockPeaqAccount::Alice, 30), (MockPeaqAccount::ParentAccount, 30)])
		.with_delegators(vec![
			(MockPeaqAccount::Bob, MockPeaqAccount::Alice, 20),
			(MockPeaqAccount::Charlie, MockPeaqAccount::Alice, 20),
		])
		.set_blocks_per_round(5)
		.build()
		.execute_with(|| {
			let mut old_stake = StakePallet::total_collator_stake();
			assert_eq!(old_stake, TotalStake { collators: 60, delegators: 40 });

			old_stake = StakePallet::total_collator_stake();
			precompiles()
				.prepare_test(
					MockPeaqAccount::Bob,
					MockPeaqAccount::EVMu1Account,
					PCall::delegator_stake_more {
						collator: Address(MockPeaqAccount::Alice.into()),
						amount: 50.into(),
					},
				)
				.expect_no_logs()
				.execute_returns(());

			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake { delegators: old_stake.delegators + 50, ..old_stake }
			);

			old_stake = StakePallet::total_collator_stake();
			precompiles()
				.prepare_test(
					MockPeaqAccount::Bob,
					MockPeaqAccount::EVMu1Account,
					PCall::delegator_stake_less {
						collator: Address(MockPeaqAccount::Alice.into()),
						amount: 50.into(),
					},
				)
				.expect_no_logs()
				.execute_returns(());
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake { delegators: old_stake.delegators - 50, ..old_stake }
			);

			old_stake = StakePallet::total_collator_stake();
			precompiles()
				.prepare_test(
					MockPeaqAccount::David,
					MockPeaqAccount::EVMu1Account,
					PCall::join_delegators {
						collator: Address(MockPeaqAccount::Alice.into()),
						amount: 50.into(),
					},
				)
				.expect_no_logs()
				.execute_returns(());

			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake { delegators: old_stake.delegators + 50, ..old_stake }
			);

			old_stake = StakePallet::total_collator_stake();
			precompiles()
				.prepare_test(
					MockPeaqAccount::David,
					MockPeaqAccount::EVMu1Account,
					PCall::delegate_another_candidate {
						collator: Address(MockPeaqAccount::ParentAccount.into()),
						amount: 60.into(),
					},
				)
				.expect_no_logs()
				.execute_returns(());

			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake { delegators: old_stake.delegators + 60, ..old_stake }
			);

			old_stake = StakePallet::total_collator_stake();
			assert_eq!(StakePallet::delegator_state(MockPeaqAccount::Charlie).unwrap().total, 20);
			precompiles()
				.prepare_test(
					MockPeaqAccount::Charlie,
					MockPeaqAccount::EVMu1Account,
					PCall::leave_delegators {},
				)
				.expect_no_logs()
				.execute_returns(());
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake { delegators: old_stake.delegators - 20, ..old_stake }
			);
			/*             let old_stake = StakePallet::total_collator_stake();
			 *             assert_eq!(StakePallet::delegator_state(8).unwrap().total, 10);
			 *             assert_ok!(StakePallet::revoke_delegation(RuntimeOrigin::signed(8),
			 * 2));             assert_eq!(
			 *                 StakePallet::total_collator_stake(),
			 *                TotalStake { delegators: old_stake.delegators - 10, ..old_stake }
			 *             );
			 *
			 *             // should immediately affect total stake because collator can't be
			 * chosen in             // active set from now on, thus delegated stake is reduced
			 *             let old_stake = StakePallet::total_collator_stake();
			 *             assert_eq!(StakePallet::candidate_pool(2).unwrap().total, 30);
			 *             assert_eq!(StakePallet::candidate_pool(2).unwrap().stake, 20);
			 *             assert_eq!(StakePallet::selected_candidates().into_inner(), vec![2,
			 * 1]);             assert_eq!(
			 *                 StakePallet::candidate_pool(2).unwrap().stake,
			 *                 StakePallet::candidate_pool(3).unwrap().stake
			 *             );
			 *
			 * assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(2)));
			 *             let old_stake = TotalStake {
			 *                 delegators: old_stake.delegators - 10,
			 *                 // total active collator stake is unchanged because number of
			 * selected candidates is                 // 2 and 2's replacement has the same self
			 * stake as 2                 collators: old_stake.collators,
			 *             };
			 *             assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1,
			 * 3]);             assert_eq!(StakePallet::total_collator_stake(), old_stake);
			 *
			 *             // shouldn't change total stake when 2 leaves
			 *             roll_to(10, vec![]);
			 *             assert_eq!(StakePallet::total_collator_stake(), old_stake);
			 *
			 * assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(2), 2));
			 *             assert_eq!(StakePallet::total_collator_stake(), old_stake);
			 */
		})
}
