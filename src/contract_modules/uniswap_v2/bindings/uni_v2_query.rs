pub use uni_v2_query::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types
)]
pub mod uni_v2_query {
    #[rustfmt::skip]
    const __ABI: &str = "[\n    {\n      \"inputs\": [\n        {\n          \"internalType\": \"contract UniswapV2Factory\",\n          \"name\": \"_uniswapFactory\",\n          \"type\": \"address\"\n        },\n        {\n          \"internalType\": \"uint256\",\n          \"name\": \"_start\",\n          \"type\": \"uint256\"\n        },\n        {\n          \"internalType\": \"uint256\",\n          \"name\": \"_stop\",\n          \"type\": \"uint256\"\n        }\n      ],\n      \"name\": \"getPairsAndReserves\",\n      \"outputs\": [\n        {\n          \"internalType\": \"address[3][]\",\n          \"name\": \"pairs\",\n          \"type\": \"address[3][]\"\n        },\n        {\n          \"internalType\": \"uint256[3][]\",\n          \"name\": \"reserves\",\n          \"type\": \"uint256[3][]\"\n        }\n      ],\n      \"stateMutability\": \"view\",\n      \"type\": \"function\"\n    }\n]";
    ///The parsed JSON ABI of the contract.
    pub static UNIV2QUERY_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(|| {
            ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid")
        });
    pub struct UniV2Query<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for UniV2Query<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for UniV2Query<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for UniV2Query<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for UniV2Query<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(UniV2Query))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> UniV2Query<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                UNIV2QUERY_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `getPairsAndReserves` (0x6a750bac) function
        pub fn get_pairs_and_reserves(
            &self,
            uniswap_factory: ::ethers::core::types::Address,
            start: ::ethers::core::types::U256,
            stop: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::std::vec::Vec<[::ethers::core::types::Address; 3]>,
                ::std::vec::Vec<[::ethers::core::types::U256; 3]>,
            ),
        > {
            self.0
                .method_hash([106, 117, 11, 172], (uniswap_factory, start, stop))
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>> for UniV2Query<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `getPairsAndReserves` function with signature `getPairsAndReserves(address,uint256,uint256)` and selector `0x6a750bac`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(
        name = "getPairsAndReserves",
        abi = "getPairsAndReserves(address,uint256,uint256)"
    )]
    pub struct GetPairsAndReservesCall {
        pub uniswap_factory: ::ethers::core::types::Address,
        pub start: ::ethers::core::types::U256,
        pub stop: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `getPairsAndReserves` function with signature `getPairsAndReserves(address,uint256,uint256)` and selector `0x6a750bac`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct GetPairsAndReservesReturn {
        pub pairs: ::std::vec::Vec<[::ethers::core::types::Address; 3]>,
        pub reserves: ::std::vec::Vec<[::ethers::core::types::U256; 3]>,
    }
}
