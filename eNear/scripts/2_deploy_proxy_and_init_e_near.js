const prompt = require('prompt-sync')();

const eNearABI = require('../artifacts/contracts/eNear.sol/eNear.json').abi

async function main() {
  const [deployer] = await ethers.getSigners()
  const deployerAddress = await deployer.getAddress()
  console.log(
    "Deploying eNear proxy with the account:",
    deployerAddress
  )

  const eNearLogicAddress = prompt('Logic address? ');
  const tokenName = prompt('Token name? ');
  const tokenSymbol = prompt('Token symbol? ');
  const nearConnector = prompt('Near connector account? ');
  const proverAddress = prompt('Prover address? ');
  const minBlockAcceptanceHeight = prompt('Min block height? ');
  const adminAddress = prompt('eNear and proxy admin? ');
  const pausedFlagValues = prompt('Paused flag values? ');

  const proxyFactory = await ethers.getContractFactory("TransparentUpgradeableProxyNear")
  const proxy = await proxyFactory.deploy(
    eNearLogicAddress,
    adminAddress,
    await new web3.eth.Contract(eNearABI).methods.init(
      tokenName,
      tokenSymbol,
      Buffer.from(nearConnector, 'utf-8'),
      proverAddress,
      minBlockAcceptanceHeight,
      adminAddress,
      pausedFlagValues
    ).encodeABI()
  )

  await proxy.deployed()

  console.log('proxy deployed at', proxy.address)

  console.log('Done')
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
