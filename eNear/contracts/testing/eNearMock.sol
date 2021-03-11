// SPDX-License-Identifier: MIT

pragma solidity 0.6.12;

import "rainbow-bridge/contracts/eth/nearprover/contracts/INearProver.sol";
import { eNear } from "../eNear.sol";

contract eNearMock is eNear {

    function mintTo(address _recipient, uint256 _amount) external {
        _mint(_recipient, _amount);
    }

    function burn(uint256 _amount) external {
        _burn(_msgSender(), _amount);
    }
}
