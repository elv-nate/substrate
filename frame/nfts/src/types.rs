// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Various basic types for use in the Nfts pallet.

use super::*;
use codec::EncodeLike;
use enumflags2::{bitflags, BitFlags};
use frame_support::{
	pallet_prelude::{BoundedVec, MaxEncodedLen},
	traits::Get,
};
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};

pub(super) type DepositBalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;
pub(super) type CollectionDetailsFor<T, I> =
	CollectionDetails<<T as SystemConfig>::AccountId, DepositBalanceOf<T, I>>;
pub(super) type ItemDetailsFor<T, I> =
	ItemDetails<<T as SystemConfig>::AccountId, DepositBalanceOf<T, I>, ApprovalsOf<T, I>>;
pub(super) type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;
pub(super) type ItemPrice<T, I = ()> = BalanceOf<T, I>;
pub(super) type ItemTipOf<T, I = ()> = ItemTip<
	<T as Config<I>>::CollectionId,
	<T as Config<I>>::ItemId,
	<T as SystemConfig>::AccountId,
	BalanceOf<T, I>,
>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CollectionDetails<AccountId, DepositBalance> {
	/// Can change `owner`, `issuer`, `freezer` and `admin` accounts.
	pub(super) owner: AccountId,
	/// Can mint tokens.
	pub(super) issuer: AccountId,
	/// Can thaw tokens, force transfers and burn tokens from any account.
	pub(super) admin: AccountId,
	/// Can freeze tokens.
	pub(super) freezer: AccountId,
	/// The total balance deposited for the all storage associated with this collection.
	/// Used by `destroy`.
	pub(super) total_deposit: DepositBalance,
	/// The total number of outstanding items of this collection.
	pub(super) items: u32,
	/// The total number of outstanding item metadata of this collection.
	pub(super) item_metadatas: u32,
	/// The total number of attributes for this collection.
	pub(super) attributes: u32,
}

/// Witness data for the destroy transactions.
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct DestroyWitness {
	/// The total number of outstanding items of this collection.
	#[codec(compact)]
	pub items: u32,
	/// The total number of items in this collection that have outstanding item metadata.
	#[codec(compact)]
	pub item_metadatas: u32,
	#[codec(compact)]
	/// The total number of attributes for this collection.
	pub attributes: u32,
}

impl<AccountId, DepositBalance> CollectionDetails<AccountId, DepositBalance> {
	pub fn destroy_witness(&self) -> DestroyWitness {
		DestroyWitness {
			items: self.items,
			item_metadatas: self.item_metadatas,
			attributes: self.attributes,
		}
	}
}

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct ItemDetails<AccountId, DepositBalance, Approvals> {
	/// The owner of this item.
	pub(super) owner: AccountId,
	/// The approved transferrer of this item, if one is set.
	pub(super) approvals: Approvals,
	/// The amount held in the pallet's default account for this item. Free-hold items will have
	/// this as zero.
	pub(super) deposit: DepositBalance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(StringLimit))]
#[codec(mel_bound(DepositBalance: MaxEncodedLen))]
pub struct CollectionMetadata<DepositBalance, StringLimit: Get<u32>> {
	/// The balance deposited for this metadata.
	///
	/// This pays for the data stored in this struct.
	pub(super) deposit: DepositBalance,
	/// General information concerning this collection. Limited in length by `StringLimit`. This
	/// will generally be either a JSON dump or the hash of some JSON which can be found on a
	/// hash-addressable global publication system such as IPFS.
	pub(super) data: BoundedVec<u8, StringLimit>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(StringLimit))]
#[codec(mel_bound(DepositBalance: MaxEncodedLen))]
pub struct ItemMetadata<DepositBalance, StringLimit: Get<u32>> {
	/// The balance deposited for this metadata.
	///
	/// This pays for the data stored in this struct.
	pub(super) deposit: DepositBalance,
	/// General information concerning this item. Limited in length by `StringLimit`. This will
	/// generally be either a JSON dump or the hash of some JSON which can be found on a
	/// hash-addressable global publication system such as IPFS.
	pub(super) data: BoundedVec<u8, StringLimit>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ItemTip<CollectionId, ItemId, AccountId, Amount> {
	/// A collection of the item.
	pub(super) collection: CollectionId,
	/// An item of which the tip is send for.
	pub(super) item: ItemId,
	/// A sender of the tip.
	pub(super) receiver: AccountId,
	/// An amount the sender is willing to tip.
	pub(super) amount: Amount,
}

// Support for up to 64 user-enabled features on a collection.
#[bitflags]
#[repr(u64)]
#[derive(Copy, Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum CollectionSetting {
	/// Disallow to transfer items in this collection.
	NonTransferableItems,
	/// Disallow to modify metadata of this collection.
	LockedMetadata,
	/// Disallow to modify attributes of this collection.
	LockedAttributes,
	/// When is set then no deposit needed to hold items of this collection.
	FreeHolding,
}

pub(super) type CollectionSettings = BitFlags<CollectionSetting>;

/// Wrapper type for `CollectionSettings` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Eq, Default, RuntimeDebug)]
pub struct CollectionConfig(pub CollectionSettings);

impl CollectionConfig {
	pub fn empty() -> Self {
		Self(BitFlags::EMPTY)
	}

	pub fn values(&self) -> CollectionSettings {
		self.0
	}
}

impl MaxEncodedLen for CollectionConfig {
	fn max_encoded_len() -> usize {
		u64::max_encoded_len()
	}
}

impl Encode for CollectionConfig {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl EncodeLike for CollectionConfig {}
impl Decode for CollectionConfig {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u64::decode(input)?;
		Ok(Self(CollectionSettings::from_bits(field as u64).map_err(|_| "invalid value")?))
	}
}

impl TypeInfo for CollectionConfig {
	type Identity = Self;

	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("BitFlags", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<CollectionSetting>()))])
			.composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("CollectionSetting")))
	}
}

// Support for up to 64 user-enabled features on an item.
#[bitflags]
#[repr(u64)]
#[derive(Copy, Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ItemSetting {
	/// Disallow transferring this item.
	NonTransferable,
	/// Disallow to modify metadata of this item.
	LockedMetadata,
	/// Disallow to modify attributes of this item.
	LockedAttributes,
}

pub(super) type ItemSettings = BitFlags<ItemSetting>;

/// Wrapper type for `ItemSettings` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Eq, Default, RuntimeDebug)]
pub struct ItemConfig(pub ItemSettings);

impl ItemConfig {
	pub fn empty() -> Self {
		Self(BitFlags::EMPTY)
	}

	pub fn values(&self) -> ItemSettings {
		self.0
	}
}

impl MaxEncodedLen for ItemConfig {
	fn max_encoded_len() -> usize {
		u64::max_encoded_len()
	}
}

impl Encode for ItemConfig {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl EncodeLike for ItemConfig {}
impl Decode for ItemConfig {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u64::decode(input)?;
		Ok(Self(ItemSettings::from_bits(field as u64).map_err(|_| "invalid value")?))
	}
}

impl TypeInfo for ItemConfig {
	type Identity = Self;

	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("BitFlags", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<ItemSetting>()))])
			.composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("ItemSetting")))
	}
}

// Support for up to 64 system-enabled features on a collection.
#[bitflags]
#[repr(u64)]
#[derive(Copy, Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum SystemFeature {
	/// Disallow trading operations.
	NoTrading,
	/// Disallow setting attributes.
	NoAttributes,
	/// Disallow transfer approvals.
	NoApprovals,
	/// Disallow atomic items swap.
	NoSwaps,
	/// Disallow public mints.
	NoPublicMints,
}

pub type SystemFeatureFlags = BitFlags<SystemFeature>;

/// Wrapper type for `SystemFeatureFlags` that implements `Codec`.
#[derive(Default, RuntimeDebug)]
pub struct SystemFeatures(pub SystemFeatureFlags);

impl MaxEncodedLen for SystemFeatures {
	fn max_encoded_len() -> usize {
		u64::max_encoded_len()
	}
}

impl Encode for SystemFeatures {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl EncodeLike for SystemFeatures {}
impl Decode for SystemFeatures {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u64::decode(input)?;
		Ok(Self(SystemFeatureFlags::from_bits(field as u64).map_err(|_| "invalid value")?))
	}
}

impl TypeInfo for SystemFeatures {
	type Identity = Self;

	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("BitFlags", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<SystemFeature>()))])
			.composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("SystemFeature")))
	}
}
