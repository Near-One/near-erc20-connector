const prompt = require('prompt-sync')();

async function main() {
  const [deployer] = await ethers.getSigners()
  const deployerAddress = await deployer.getAddress()
  console.log(
    "Deploying eNear with the account:",
    deployerAddress
  )

  const tokenName = prompt('Token name? ');
  const tokenSymbol = prompt('Token symbol? ');
  const minterAndBurner = prompt('Minter and burner? ');

  const eNearFactory = await ethers.getContractFactory("eNear")
  const eNear = await eNearFactory.deploy(
    tokenName,
    tokenSymbol,
    minterAndBurner
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
