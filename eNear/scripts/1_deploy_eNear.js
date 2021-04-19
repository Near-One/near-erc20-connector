const prompt = require('prompt-sync')()

async function main() {
  const [deployer] = await ethers.getSigners()
  const deployerAddress = await deployer.getAddress()
  console.log(
    "Deploying eNear contract with the account:",
    deployerAddress
  )

  const tokenName = prompt('Token name? ');
  const tokenSymbol = prompt('Token symbol? ');
  const nearConnector = prompt('Near connector account? ');
  const proverAddress = prompt('Prover address? ');
  const minBlockAcceptanceHeight = prompt('Min block height? ');
  const adminAddress = prompt('eNear and proxy admin? ');
  const pausedFlagValues = prompt('Paused flag values? ');

  const eNearFactory = await ethers.getContractFactory("eNear")
  const eNear = await eNearFactory.deploy(
    tokenName,
    tokenSymbol,
    Buffer.from(nearConnector, 'utf-8'),
    proverAddress,
    minBlockAcceptanceHeight,
    adminAddress,
    pausedFlagValues
  )

  await eNear.deployed()

  console.log('eNear deployed at', eNear.address)

  console.log('Done')
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
