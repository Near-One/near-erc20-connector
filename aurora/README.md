# Overview

Smart contract for transferring NEAR from Aurora to Ethereum. This is done by making an XCC call to the bridge contract with required wNear amount attached.

**Attention**: we shall ourselves make the first transfer after deploying the contract in order to provide 2 NEAR required to create an account for XCC calls.

## Working with the project

Install dependencies
```
yarn
```

Build
```
yarn build
```

Deploy
```
yarn deploy:testnet
yarn deploy:mainnet
```

Verify
```
yarn verify:testnet {contract_id}
yarn verify:mainnet {contract_id}
```

Upgrade
```
yarn upgrade:testnet
yarn upgrade:mainnet
```

Admin withdraw
```
yarn hardhat withdraw --environment testnet --network testnet_aurora --recipient <recipient> --amount <amount>
yarn hardhat withdraw --environment mainnet --network testnet_aurora --recipient <recipient> --amount <amount>
```