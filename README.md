# near-erc20-connector
A connector for the Rainbow Bridge that allows sending $NEAR to Ethereum which is then minted as an ERC-20 token (eNEAR)

## eNear Ethereum smart contracts

This is a hardhat + yarn project.

To install the dependencies:

```
yarn
```

Compile contracts using:

```
yarn compile
```

To run tests:

```
yarn test
```

or

```
yarn coverage
```

### Hardhat scripts

The eNear contract can be deployed with the following command:

```
yarn hardhat deploy-enear
```

This command takes the following command line arguments:

- --token-name - Token Name for ERC20
- --token-symbol - Token Symbol for ERC20
- --near-connector - Near Connector Account ID
- --near-prover - Near on ETH prover address
- --block-height - Min block acceptance height
- --eth-admin - Eth admin controlled address
- --paused-flags - Paused flags
