require('dotenv').config();
require("@nomiclabs/hardhat-waffle");
require("@nomiclabs/hardhat-truffle5");
require('solidity-coverage');
require('hardhat-gas-reporter');
require('@nomiclabs/hardhat-solhint');
require('./scripts/1_deploy_eNear');
require('./scripts/2_pause_e_near_method');

function rpc_endpoint(network) {
  return process.env[`${network.toUpperCase()}_RPC_ENDPOINT`] || '';
}

function private_key(network) {
  return process.env[`${network.toUpperCase()}_PRIVATE_KEY`] || '0x0000000000000000000000000000000000000000000000000000000000000000'
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
      accounts: [private_key('ropsten')]
    },
    mainnet: {
      // MAINNET_RPC_ENDPOINT
      url: rpc_endpoint('mainnet'),
      // MAINNET_PRIVATE_KEY
      accounts: [private_key('mainnet')]
    }
  }
};
