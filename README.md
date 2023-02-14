# axelar-cgp-near

<p align="middle">
    <a href="https://axelar.network/brand" target="blank"><img src="./tech-logos/axelar_logo.png" width="200" alt="Axelar Logo"/></a>
    &nbsp;
    &nbsp;
    &nbsp;
    &nbsp;
    &nbsp;
    <a href="https://near.org/" target="blank"><img src="./tech-logos/near_logo.png" width="200" alt="Near Logo" /></a>
</p>

# Exploring The Code

1. The Axelar Authentication and Gateway smart-contract code lives in the `/contract` folder.
2. The example smart-contract that supports communication with Gateway lives in the `/near-axelar-contract-call-example` folder.
3. Test contract using: `npm test`, this will run the tests in `integration-tests` directory.

# Quick Start

Install dependencies:

    npm install

Build contract:

    npm run build

Build and deploy your contract to TestNet with a temporary dev account:

    npm run deploy

Test your contract:

    npm run test

# Deploy

Every smart contract in NEAR has its [own associated account][near accounts].
When you run `npm run deploy`, your smart contract gets deployed to the live NEAR TestNet with a temporary dev account.
When you're ready to make it permanent, here's how:

## Step 0: Install near-cli (optional)

[near-cli] is a command line interface (CLI) for interacting with the NEAR blockchain. It was installed to the local `node_modules` folder when you ran `npm install`, but for best ergonomics you may want to install it globally:

    npm install --global near-cli

Or, if you'd rather use the locally-installed version, you can prefix all `near` commands with `npx`

Ensure that it's installed with `near --version` (or `npx near --version`)

## Step 1: Create an account for the contract

Each account on NEAR can have at most one contract deployed to it. If you've already created an account such as `your-name.testnet`, you can deploy your contract to `near-blank-project.your-name.testnet`. Assuming you've already created an account on [NEAR Wallet], here's how to create `near-blank-project.your-name.testnet`:

1. Authorize NEAR CLI, following the commands it gives you:

   near login

2. Create a subaccount (replace `YOUR-NAME` below with your actual account name):

   near create-account near-blank-project.YOUR-NAME.testnet --masterAccount YOUR-NAME.testnet

## Step 2: deploy the contract

Use the CLI to deploy the contract to TestNet with your account ID.
Replace `PATH_TO_WASM_FILE` with the `wasm` that was generated in `contract` build directory.

    near deploy --accountId near-blank-project.YOUR-NAME.testnet --wasmFile PATH_TO_WASM_FILE
