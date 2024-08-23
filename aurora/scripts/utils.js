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

async function upgrade({
  signer,
  proxyAddress,
  auroraSdkAddress,
  auroraUtilsAddress,
}) {
  const NearBridgeContract = (
    await ethers.getContractFactory("NearBridge", {
      libraries: {
        AuroraSdk: auroraSdkAddress,
        Utils: auroraUtilsAddress,
      },
    })
  ).attach(proxyAddress).connect(signer);

  console.log(`Owner: ${await NearBridgeContract.owner()}`);
  console.log(
    "Current implementation address:",
    await upgrades.erc1967.getImplementationAddress(proxyAddress)
  );
  console.log("Upgrade NearBridge contract, proxy address", proxyAddress);
  const proxy = await upgrades.upgradeProxy(proxyAddress, NearBridgeContract, {
    unsafeAllowLinkedLibraries: true,
    gasLimit: 6000000,
  });
  await proxy.waitForDeployment();

  console.log(
    "NearBridgeContract impl deployed to: ",
    await upgrades.erc1967.getImplementationAddress(await proxy.getAddress())
  );
}

async function withdraw({
  recipientAddress,
  amount,
  signer,
  wNearAccountId,
  proxyAddress,
  auroraSdkAddress,
  auroraUtilsAddress,
}) {
  const NearBridgeContract = (
    await ethers.getContractFactory("NearBridge", {
      libraries: {
        AuroraSdk: auroraSdkAddress,
        Utils: auroraUtilsAddress,
      },
    })
  )
    .attach(proxyAddress)
    .connect(signer);

  let tx = await NearBridgeContract.withdrawFromImplicitNearAccount(
    "aurora",
    wNearAccountId,
    recipientAddress,
    amount
  );
  console.log(tx);
  let receipt = await tx.wait();
  console.log(receipt);
}

exports.deploy = deploy;
exports.upgrade = upgrade;
exports.withdraw = withdraw;