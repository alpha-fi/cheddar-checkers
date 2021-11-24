NEAR Checkers

Compilation of [Rust code "Rusty checkers"](https://github.com/dboone/rusty-checkers), [Plain Javascript UI](https://github.com/codethejason/checkers) and some glue from [near-api-js](https://docs.near.org/docs/api/javascript-library).

[DEMO // NEAR Mainnet](https://checkers.nearspace.info/)

User Interface: https://github.com/zavodil/near-checkers-ui

Some Rules:
- Set a bid and join waiting list or select an available player to start the game.
- Winner gets a bank
- Invite a friend to get a 10% referral bonus from his rewards.
- Hold shift button (or check a checkbox) to perform a double jump. Release a shift button before a final move.
- If you spent more than an hour, your opponent may stop the game and get the reward.
- Service fee is 10%, referral reward is half of the service fee.
- Various game stats are storing onchain

Near-rusty-chekers Smart Contract
==================

A [smart contract] written in [Rust] for an app initialized with [create-near-app]


Quick Start
===========

Before you compile this code, you will need to install Rust with [correct target]


Exploring The Code
==================

1. The main smart contract code lives in `src/lib.rs`. You can compile it with
   the `./compile` script.
2. Tests: You can run smart contract tests with the `./test` script. This runs
   standard Rust tests using [cargo] with a `--nocapture` flag so that you
   can see any debug info you print to the console.


  [smart contract]: https://docs.near.org/docs/develop/contracts/overview
  [Rust]: https://www.rust-lang.org/
  [create-near-app]: https://github.com/near/create-near-app
  [correct target]: https://github.com/near/near-sdk-rs#pre-requisites
  [cargo]: https://doc.rust-lang.org/book/ch01-03-hello-cargo.html
