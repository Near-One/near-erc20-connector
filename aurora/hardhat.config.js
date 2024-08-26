require("dotenv").config();
require("@openzeppelin/hardhat-upgrades");
require("@nomicfoundation/hardhat-verify");

const AURORA_PRIVATE_KEY = process.env.AURORA_PRIVATE_KEY || "11".repeat(32);
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
    },
  },
  etherscan: {
    apiKey: {
      mainnet_aurora: `${ETHERSCAN_API_KEY}`,
      testnet_aurora: `${ETHERSCAN_API_KEY}`,
    },
    customChains: [
      {
        network: "mainnet_aurora",
        chainId: 1313161554,
        urls: {
          apiURL: "https://old.explorer.aurora.dev/api",
          browserURL: "https://explorer.mainnet.aurora.dev",
        },
      },
      {
        network: "testnet_aurora",
        chainId: 1313161555,
        urls: {
          apiURL: "https://explorer.testnet.aurora.dev/api",
          browserURL: "https://explorer.testnet.aurora.dev",
        },
      },
    ],
  },
};

task("deploy", "Deploy bridge contract")
  .addParam("environment", "Config file name without extension")
  .setAction(async (taskArgs, hre) => {
    const { deploy } = require("./scripts/utils.js");
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

task("deployImpl", "Deploy implementation bridge contract")
  .addParam("environment", "Config file name without extension")
  .setAction(async (taskArgs, hre) => {
    const { deployImplementation } = require("./scripts/utils.js");
    const config = require(`./scripts/aurora_${taskArgs.environment}.params.json`);

    await hre.run("compile");
    await deployImplementation({
      wNearAddress: config.wNearAddress,
      eNearAccountId: config.eNearAccountId,
      auroraSdkAddress: config.auroraSdkAddress,
      auroraUtilsAddress: config.utilsAddress,
    });
  });

task("upgrade", "Upgrade bridge contract")
  .addParam("environment", "Config file name without extension")
  .setAction(async (taskArgs, hre) => {
    const { upgrade } = require("./scripts/utils.js");
    const [signer] = await hre.ethers.getSigners();
    const config = require(`./scripts/aurora_${taskArgs.environment}.params.json`);

    await hre.run("compile");
    await upgrade({
      signer,
      proxyAddress: config.proxyAddress,
      auroraSdkAddress: config.auroraSdkAddress,
      auroraUtilsAddress: config.utilsAddress,
    });
  });


task("withdraw", "Withdraw from implicit near account")
  .addParam("recipient", "Recipient address")
  .addParam("amount", "Amount to withdraw")
  .addParam("environment", "Config file name without extension")
  .setAction(async (taskArgs, hre) => {
    const { withdraw } = require("./scripts/utils.js");
    const [signer] = await hre.ethers.getSigners();
    const config = require(`./scripts/aurora_${taskArgs.environment}.params.json`);

    await hre.run("compile");
    await withdraw({
      recipientAddress: taskArgs.recipient,
      amount: taskArgs.amount,
      signer,
      wNearAccountId: config.wNearAccountId,
      proxyAddress: config.proxyAddress,
      auroraSdkAddress: config.auroraSdkAddress,
      auroraUtilsAddress: config.utilsAddress,
    });
  });
