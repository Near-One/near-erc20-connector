async function main() {
  const [deployer] = await ethers.getSigners()
  const deployerAddress = await deployer.getAddress()
  console.log(
    "Deploying eNear logic contract with the account:",
    deployerAddress
  )

  const eNearFactory = await ethers.getContractFactory("eNear")
  const eNear = await eNearFactory.deploy()

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
