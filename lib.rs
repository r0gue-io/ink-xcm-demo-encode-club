#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod xcm_complete {
    use ink::{
        env::Error as EnvError,
        xcm::prelude::*,
        prelude::vec::Vec,
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct XcmComplete;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum RuntimeError {
        XcmExecuteFailed,
        XcmSendFailed,
    }

    impl From<EnvError> for RuntimeError {
        fn from(e: EnvError) -> Self {
            use ink::env::ReturnErrorCode;
            match e {
                EnvError::ReturnError(ReturnErrorCode::XcmExecutionFailed) => {
                    RuntimeError::XcmExecuteFailed
                }
                EnvError::ReturnError(ReturnErrorCode::XcmSendFailed) => {
                    RuntimeError::XcmSendFailed
                }
                _ => panic!("Unexpected error from `pallet-contracts`."),
            }
        }
    }

    impl XcmComplete {
        /// The constructor is `payable`, so that during instantiation it can be given
        /// some tokens that will be further transferred when transferring funds through
        /// XCM.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn transfer_through_xcm(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) -> Result<(), RuntimeError> {
            let asset: Asset = (Location::parent(), value).into();
            let beneficiary = AccountId32 {
                network: None,
                id: *receiver.as_ref(),
            };

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution(asset.clone(), Unlimited)
                .deposit_asset(asset.into(), beneficiary.into())
                .build();

            self.env()
                .xcm_execute(&VersionedXcm::V4(message))
                .map_err(Into::into)
        }

        #[ink(message)]
        pub fn generic_execute(
            &mut self,
            encoded_extrinsic: Vec<u8>,
            fee_max: Balance,
            ref_time: u64,
            proof_size: u64,
        ) -> Result<XcmHash, RuntimeError> {
            let asset: Asset = (Here, fee_max).into();

            let dest = Location::parent().into_versioned();

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution(asset.clone(), Unlimited)
                .transact(
                    OriginKind::SovereignAccount,
                    Weight::from_parts(ref_time, proof_size),
                    encoded_extrinsic.into(),
                )
                .build();


            self.env().xcm_send(&dest, &VersionedXcm::V4(message))
                .map_err(Into::into)
        }

        #[ink(message)]
        pub fn generic_execute_ah(
            &mut self,
            encoded_extrinsic: Vec<u8>,
            fee_max: crate::xcm_complete::Balance,
            ref_time: u64,
            proof_size: u64,
        ) -> Result<XcmHash, crate::xcm_complete::RuntimeError> {
            let asset: Asset = (Location::parent(), fee_max).into();
            let ah = Junctions::from([Parachain(1000)]);
            let dest: Location = Location { parents: 1, interior: ah};

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution(asset.clone(), Unlimited)
                .transact(
                    OriginKind::SovereignAccount,
                    Weight::from_parts(ref_time, proof_size),
                    encoded_extrinsic.into(),
                )
                .build();


            self.env().xcm_send(&VersionedLocation::V4(dest), &VersionedXcm::V4(message))
                .map_err(Into::into)
        }

        #[ink(message)]
        pub fn generic_execute_local(
            &mut self,
            encoded_extrinsic: Vec<u8>,
            fee_max: crate::xcm_complete::Balance,
            ref_time: u64,
            proof_size: u64,
        ) -> Result<(), crate::xcm_complete::RuntimeError> {
            let asset: Asset = (Location::parent(), fee_max).into();

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution(asset.clone(), Unlimited)
                .transact(
                    OriginKind::SovereignAccount,
                    Weight::from_parts(ref_time, proof_size),
                    encoded_extrinsic.into(),
                )
                .build();

            self.env()
                .xcm_execute(&VersionedXcm::V4(message))
                .map_err(Into::into)
        }

        /// Transfer some funds on the relay chain via XCM from the contract's derivative
        /// account to the caller's account.
        ///
        /// Fails if:
        ///  - called in the off-chain environment
        ///  - the chain is not configured to support XCM
        ///  - the XCM program executed failed (e.g contract doesn't have enough balance)
        #[ink(message)]
        pub fn send_funds(
            &mut self,
            value: Balance,
            fee: Balance,
        ) -> Result<XcmHash, RuntimeError> {
            let destination: Location = Parent.into();
            let asset: Asset = (Here, value).into();
            let beneficiary = AccountId32 {
                network: None,
                id: *self.env().caller().as_ref(),
            };

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution((Here, fee).into(), WeightLimit::Unlimited)
                .deposit_asset(asset.into(), beneficiary.into())
                .build();

            let hash = self.env().xcm_send(
                &VersionedLocation::V4(destination),
                &VersionedXcm::V4(message),
            )?;

            Ok(hash)
        }

        #[ink(message)]
        pub fn send_funds_ah(
            &mut self,
            value: crate::xcm_complete::Balance,
            fee: crate::xcm_complete::Balance,
        ) -> Result<XcmHash, crate::xcm_complete::RuntimeError> {
            let ah = Junctions::from([Parachain(1000)]);
            let destination: Location = Location { parents: 1, interior: ah};
            let asset: Asset = (Location::parent(), value).into();
            let fee_asset: Asset = (Location::parent(), fee).into();
            let beneficiary = AccountId32 {
                network: None,
                id: *self.env().caller().as_ref(),
            };

            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone().into())
                .buy_execution(fee_asset.into(), WeightLimit::Unlimited)
                .deposit_asset(asset.into(), beneficiary.into())
                .build();

            let hash = self.env().xcm_send(
                &VersionedLocation::V4(destination),
                &VersionedXcm::V4(message),
            )?;

            Ok(hash)
        }

    }
}
