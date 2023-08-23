use ethers::prelude::*;

pub const EXECUTOR_ADDRESS: &str = "0x0";
pub const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub const SYNC_TOPIC: &str = "1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1";


// CFMMS
pub const UNISWAP_V2: [(&str, &str, &str, u32); 2] = [
    (
        "0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F",
        "0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac",
        "0xe18a34eb0e04b04f7a0ac29a6e80748dca96319b42c54d679cb821dca90c6303",
        9970,
    ),
    (
        "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",
        "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f",
        "0x96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f",
        9970,
    ),
];
pub const UNISWAP_V3: &str = "None";

abigen!(UniV2Router, "src/abi/UniV2Router.json");
abigen!(UniV2Factory, "src/abi/UniV2Factory.json");
abigen!(UniV2DataQuery, "src/abi/UniV2Query.json");
