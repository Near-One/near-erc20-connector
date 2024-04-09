const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

module.exports = buildModule("Bridge", (m) => {
  const wNearAddress = m.getParameter("wNearAddress");
  const eNearAccountId = m.getParameter("eNearAccountId");

  const auroraSdk = m.contractAt("AuroraSdk", m.getParameter("auroraSdkAddress"));
  const utils = m.contractAt("Utils", m.getParameter("utilsAddress"));

  const bridge = m.contract(
    "NearBridge",
    [],
    {
      libraries: { AuroraSdk: auroraSdk, Utils: utils }
    });

  m.call(bridge, "initialize", [wNearAddress, eNearAccountId]);

  return { bridge };
});
