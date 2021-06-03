require('dotenv').config();
require("@nomiclabs/hardhat-waffle");
require("@nomiclabs/hardhat-truffle5");
require("@nomiclabs/hardhat-etherscan");
require('solidity-coverage');
require('hardhat-gas-reporter');
require('@nomiclabs/hardhat-solhint');
require('./scripts/1_deploy_eNear');
require('./scripts/2_pause_e_near_method');

/**
 * Read env variable looking for rpc endpoint to be used. Set this variable using:
 *
 * export MAINNET_RPC_ENDPOINT=...
 * export ROPSTEN_RPC_ENDPOINT=...
 *
 * @param {string} network
 * @returns {string[]} List of private keys to be used to deploy and submit.
 */
function rpc_endpoint(network) {
  return process.env[`${network.toUpperCase()}_RPC_ENDPOINT`] || '';
}

/**
 * Read env variable looking for private keys to be used. Set this variable using:
 *
 * export MAINNET_PRIVATE_KEY=...
 * export ROPSTEN_PRIVATE_KEY=...
 *
 * @param {string} network
 * @returns {string[]} List of private keys to be used to deploy and submit.
 */
function accounts(network) {
  let result = process.env[`${network.toUpperCase()}_PRIVATE_KEY`];
  if (result === undefined) {
    result = [];
  } else {
    result = [result];
  }
  return result;
}

module.exports = {
  solidity: {
    version: "0.6.12",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      }
    }
  },
  gasReporter: {
    currency: 'USD',
    enabled: false,
    gasPrice: 50
  },
  networks: {
    // Arguments are taken from env variables
    // See here for more details: https://hardhat.org/tutorial/deploying-to-a-live-network.html
    ropsten: {
      // ROPSTEN_RPC_ENDPOINT
      url: rpc_endpoint('ropsten'),
      // ROPSTEN_PRIVATE_KEY
      accounts: accounts('ropsten')
    },
    mainnet: {
      // MAINNET_RPC_ENDPOINT
      url: rpc_endpoint('mainnet'),
      // MAINNET_PRIVATE_KEY
      accounts: accounts('mainnet')
    }
  },
  etherscan: {
    apiKey: process.env['ETHERSCAN_API_KEY'] || ''
  }
};
