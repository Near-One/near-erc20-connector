const { ethers, upgrades } = require("hardhat");

async function deploy({
    wNearAddress,
    eNearAccountId,
    auroraSdkAddress,
    auroraUtilsAddress,
  }) {
    const NearBridgeContract = await ethers.getContractFactory("NearBridge", {
        libraries: {
          AuroraSdk: auroraSdkAddress,
          Utils: auroraUtilsAddress,
        },
    });
  
    let proxy = await upgrades.deployProxy(
      NearBridgeContract,
      [wNearAddress, eNearAccountId],
      {
        unsafeAllowLinkedLibraries: true,
      },
    );
    await proxy.waitForDeployment();
  
    console.log("Bridge proxy deployed to: ", await proxy.getAddress());
    console.log(
      "Bridge impl deployed to: ",
      await upgrades.erc1967.getImplementationAddress(await proxy.getAddress()),
    );
  }

exports.deploy = deploy;