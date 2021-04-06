const prompt = require('prompt-sync')()
const Web3 = require('web3')
const eNearABI = require('../artifacts/contracts/eNear.sol/eNear.json').abi
const proxyABI = require('../artifacts/contracts/TransparentUpgradeableProxyNear.sol/TransparentUpgradeableProxyNear.json').abi

async function main() {
  const [deployer] = await ethers.getSigners()
  const deployerAddress = await deployer.getAddress()
  console.log(
    "Upgrading eNear proxy with the account:",
    deployerAddress
  )

  const eNearLogicAddress = prompt('New logic address? ')
  const proxyAddress = prompt('Proxy address? ')
  const tokenName = prompt('Token name? ');
  const tokenSymbol = prompt('Token symbol? ');
  const nearConnector = prompt('Near connector account? ');
  const proverAddress = prompt('Prover address? ');
  const minBlockAcceptanceHeight = prompt('Min block height? ');
  const adminAddress = prompt('eNear and proxy admin? ');
  const pausedFlagValues = prompt('Paused flag values? ');

  const proxy = new ethers.Contract(
    proxyAddress,
    proxyABI,
    deployer
  )

  await proxy.upgradeToAndCall(
    eNearLogicAddress,
    await new Web3.eth.Contract(eNearABI).methods.init(
      tokenName,
      tokenSymbol,
      Buffer.from(nearConnector, 'utf-8'),
      proverAddress,
      minBlockAcceptanceHeight,
      adminAddress,
      pausedFlagValues
    ).encodeABI()
  )

  console.log('Done')
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
