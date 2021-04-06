const prompt = require('prompt-sync')()
const adminControlledABI = require('../artifacts/contracts/AdminControlled.sol/AdminControlled.json').abi

async function main() {
  const [deployer] = await ethers.getSigners()
  const deployerAddress = await deployer.getAddress()
  console.log(
    "Pause eNear method with the account:",
    deployerAddress
  )

  const flags = prompt('Paused flag values? ')
  const eNearProxyAddress = prompt('eNear proxy address? ')

  const eNear = new ethers.Contract(
    eNearProxyAddress,
    adminControlledABI,
    deployer
  )

  await eNear.adminPause(flags)

  console.log('Done')
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
