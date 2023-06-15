# near-erc20-connector
A connector for the Rainbow Bridge that allows sending $NEAR to Ethereum which is then minted as an ERC-20 token (eNEAR)

## eNear Ethereum smart contracts

Go to eNear directory:

```
cd eNear
```

This is a Hardhat + yarn project.

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
---
### **Fee-integration in Near-Bridge**

This implements fee for transfers of eNear from `Near -> Ethereum` and `Ethereum -> Near`

* **Fee-Setters**: {Only callable by owner or contract itself}
  * `set_transfer_fee_percentage`: setter to set the fee-percentage for bi-directional transfers of Near. It has a **6** decimal precision.
    * *For-example*: if fee-percentage to be set is 10% for both eth-to-near and near-to-eth than values to function parameter is 0.1 * 10^6 ie. 10^5.
  * `set_deposit_fee_bounds`: setter to set the fee-bounds for deposit ie. transfer from near -> ethereum.
  * `set_withdraw_fee_bounds`: setter to set the fee-bound for withdraw ie. transfer of eNEAR(Erc-20) from ethereum -> Near.
  * **NOTE**: 
    * Default value of fees is 0.
    * Since 1-NEAR = 10^24 yocto, so fee bounds is to be set in this consideration. For-example to set bounds of {1, 5} NEARs, lower-bound: 10^24 (1-NEAR) and upper-bound: 5 * 10^24 (5-NEARs)
<br>
* **Fee-Getters**: {publicly available}
  * `get_transfer_fee_percentage`: returns transfer-fee-percentage for both eth-to-near and near-to-eth. Default: 0.
  * `get_deposit_fee_bounds`: returns deposit {near -> eth} fee-bounds. Default returns 0.
  * `get_withdraw_fee_bounds`: returns withdraw {eth -> near} fee-bounds. Default: 0.
  * `get_accumulated_fee_amount`: returns claimable fee-amount accumulated.
  <br>
