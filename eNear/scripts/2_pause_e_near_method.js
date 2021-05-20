let adminControlledABI;

try {
    adminControlledABI = require('../artifacts/contracts/eNear.sol/eNear.json').abi
} catch (e) {}

task("pause-enear", "Pauses any method on a deployed eNear contract")
  .addParam('eNearContractAddress', 'Address of deployed eNear contract')
  .addParam('pausedFlags', 'Bitwise value specifying which functions are paused')
  .setAction(async taskArgs => {
    const {
      eNearContractAddress,
      pausedFlags
    } = taskArgs

    if (adminControlledABI === undefined) {
      console.log("Compile contract first with `yarn compile`");
      return;
    }

    const [deployer] = await ethers.getSigners()
    const deployerAddress = await deployer.getAddress()
    console.log(
      "Pause eNear method(s) with the account:",
      deployerAddress
    )

    const eNear = new ethers.Contract(
      eNearContractAddress,
      adminControlledABI,
      deployer
    )

    await eNear.adminPause(pausedFlags)

    console.log('Done')
  })

