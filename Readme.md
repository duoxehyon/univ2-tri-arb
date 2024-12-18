# Univ2-Tri-Arb
This code still works! Just git clone the repository and use `cargo run -r`. If you want to check token fees or ensure the pairs are valid, just run `cargo run -r`, This will make a fresh db.json file with all the taxes collected and valid pairs, else use `cargo run -r load`

### Use case
For learning and reusing components such as a uni v2 token tax checker, a generalized framework for arbitrage, etc...

This code works on all EVM blockchains, including L2s where I used to run it, specifically on Arbitrum. I've removed the executor because all optimization alphas for L2s is are the endpoints: sequencer TX feed and execution.

![example](https://github.com/duoxehyon/univ2-tri-arb/blob/main/image.png)

