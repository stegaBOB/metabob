![Crates.io](https://img.shields.io/crates/v/metabob)
![Crates.io](https://img.shields.io/crates/d/metabob)
![Crates.io](https://img.shields.io/crates/l/metabob)

# Metabob

The Metaplex NFT-standard assortment of tools for very specific tasks that are unrelated. 

*Inspired by [Metaboss](https://metaboss.rs)*

*~~Some stuff~~ Many things literally taken directly from Metaboss shhh*

---

## Installation

### From crates.io:
```bash
cargo install metabob
metabob --help
```
---
## Docs (sorta)

So... you want to sign a bunch of nfts, do ya? Like you want to sign EVERY possible NFT that you could sign? If so, you're in the right place. 

Here's how to do it: 

1. Follow the installation instructions
2. Have the keypair file somewhere close by...
3. Run the following command (it's only one command):

```bash
metabob -t 1000 -r https://ssc-dao.genesysgo.net metadata sign_all -k ~/keypair_path/keypair.json
```

The `https://ssc-dao.genesysgo.net` can be replaced by your RPC and the `~/keypair_path/keypair.json` should be replaced with the path to your filesystem wallet keypair file.

This may take a while to run. If it's timing out, try increasing the `-t 1000` to a higher number. 

---

## Contact
Email: sammy@stegabob.com

Twitter: [@stegaBOB](https://twitter.com/stegabob)

Discord: @stegaBOB#0001
