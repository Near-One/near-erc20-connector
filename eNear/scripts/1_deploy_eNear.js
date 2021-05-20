task("deploy-enear", "Deploys eNear to the desired network")
  .addParam('tokenName', 'Token Name for ERC20')
  .addParam('tokenSymbol', 'Token Symbol for ERC20')
  .addParam('nearConnector', 'Near Connector Account ID')
  .addParam('nearProver', 'Near on ETH prover address')
  .addParam('blockHeight', 'Min block acceptance height')
  .addParam('ethAdmin', 'Eth admin controlled address')
  .addParam('pausedFlags', 'Paused flags')
  .setAction(async taskArgs => {
    const {
      tokenName,
      tokenSymbol,
      nearConnector,
      nearProver,
      blockHeight,
      ethAdmin,
      pausedFlags
    } = taskArgs

    const [deployer] = await ethers.getSigners()
    const deployerAddress = await deployer.getAddress()
    console.log(
      "Deploying eNear contract with the account:",
      deployerAddress
    )

    const eNearFactory = await ethers.getContractFactory("eNear")
    const eNear = await eNearFactory.deploy(
      tokenName,
      tokenSymbol,
      Buffer.from(nearConnector, 'utf-8'),
      nearProver,
      blockHeight,
      ethAdmin,
      pausedFlags
    )

    await eNear.deployed()

    console.log('eNear deployed at', eNear.address)
    console.log('Done')
  })
