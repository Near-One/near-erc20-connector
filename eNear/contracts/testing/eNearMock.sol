// SPDX-License-Identifier: MIT

pragma solidity ^0.8;

import "rainbow-bridge-sol/nearprover/contracts/INearProver.sol";
import { eNear } from "../eNear.sol";

contract eNearMock is eNear {

    constructor(
        string memory _tokenName,
        string memory _tokenSymbol,
        bytes memory _nearConnector,
        INearProver _prover,
        uint64 _minBlockAcceptanceHeight,
        address _admin,
        uint256 _pausedFlags
    ) eNear(_tokenName, _tokenSymbol, _nearConnector, _prover, _minBlockAcceptanceHeight, _admin, _pausedFlags)
    {
    }

    function mintTo(address _recipient, uint256 _amount) external {
        _mint(_recipient, _amount);
    }
}
