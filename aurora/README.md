# Overview

Smart contract for transferring NEAR from Aurora to Ethereum. This is done by making an XCC call to the bridge contract with required wNear amount attached.

**Attention**: we shall ourselves make the first transfer after deploying the contract in order to provide 2 NEAR required to create an account for XCC calls.

## Working with the project

Install dependencies
```
npm i
```

Build
```
npx hardhat compile
```

Deploy
```
npx hardhat ignition deploy ignition/modules/Bridge.js --network {} --parameters {insert_params_file}
```