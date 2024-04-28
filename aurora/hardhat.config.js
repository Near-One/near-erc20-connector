require("dotenv").config();
require('@openzeppelin/hardhat-upgrades');
require("@nomicfoundation/hardhat-verify");

const AURORA_PRIVATE_KEY = process.env.AURORA_PRIVATE_KEY || '11'.repeat(32);
const ETHERSCAN_API_KEY = process.env.ETHERSCAN_API_KEY;

module.exports = {
  solidity: "0.8.24",
  networks: {
    testnet_aurora: {
      url: "https://testnet.aurora.dev",
      accounts: [`0x${AURORA_PRIVATE_KEY}`],
      chainId: 1313161555,
    },
    mainnet_aurora: {
      url: "https://mainnet.aurora.dev",
      accounts: [`0x${AURORA_PRIVATE_KEY}`],
      chainId: 1313161554,
    }
  },
  etherscan: {
    apiKey: {
      auroraMainnet: `${ETHERSCAN_API_KEY}`,
      auroraTestnet: `${ETHERSCAN_API_KEY}`,
    },
  }
};

task("deploy", "Deploy bridge contract")
  .addParam("environment", "Config file name without extension")
  .setAction(async (taskArgs, hre) => {
    const { deploy } = require("./scripts/deploy.js");
    const [signer] = await hre.ethers.getSigners();
    const config = require(`./scripts/aurora_${taskArgs.environment}.params.json`);

    await hre.run("compile");
    await deploy({
      wNearAddress: config.wNearAddress,
      eNearAccountId: config.eNearAccountId,
      auroraSdkAddress: config.auroraSdkAddress,
      auroraUtilsAddress: config.utilsAddress,
    });
  });
