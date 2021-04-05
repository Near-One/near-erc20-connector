// SPDX-License-Identifier: MIT

pragma solidity 0.6.12;

import "@openzeppelin/contracts/proxy/TransparentUpgradeableProxy.sol";

contract TransparentUpgradeableProxyNear is TransparentUpgradeableProxy {
    constructor(address _logic, address admin_, bytes memory _data) public
    TransparentUpgradeableProxy(_logic, admin_, _data) {}
}
