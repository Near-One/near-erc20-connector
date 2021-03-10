// SPDX-License-Identifier: MIT

pragma solidity 0.6.12;

import "rainbow-bridge/contracts/eth/nearbridge/contracts/AdminControlled.sol";
import "rainbow-bridge/contracts/eth/nearprover/contracts/ProofDecoder.sol";
import "rainbow-bridge/contracts/eth/nearbridge/contracts/Borsh.sol";
import "rainbow-bridge/contracts/eth/nearbridge/contracts/AdminControlled.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { Bridge, INearProver } from "./Bridge.sol";

contract eNear is ERC20, Bridge, AdminControlled {

    uint constant UNPAUSED_ALL = 0;
    uint constant PAUSED_FINALISE_FROM_NEAR = 1 << 0;
    uint constant PAUSED_XFER_TO_NEAR = 1 << 1;

    event TransferToNearInitiated (
        address indexed sender,
        uint256 amount,
        string accountId
    );

    event NearToEthTransferFinalised (
        uint128 amount,
        address recipient
    );

    struct BridgeResult {
        uint128 amount;
        address token;
        address recipient;
    }

    constructor(
        string memory _tokenName,
        string memory _tokenSymbol,
        bytes memory nearTokenFactory,
        INearProver prover,
        address _admin,
        uint pausedFlags
    )
    ERC20(_tokenName, _tokenSymbol)
    Bridge(nearTokenFactory, prover)
    AdminControlled(_admin, pausedFlags)
    public
    {
        // set decimals to 24 to mirror yocto Near
        _setupDecimals(24);
    }

    function finaliseNearToEthTransfer(bytes memory proofData, uint64 proofBlockHeight)
    external pausable (PAUSED_FINALISE_FROM_NEAR) {
        ProofDecoder.ExecutionStatus memory status = _parseAndConsumeProof(proofData, proofBlockHeight);
        BridgeResult memory result = _decodeBridgeResult(status.successValue);

        _mint(result.recipient, result.amount);

        emit NearToEthTransferFinalised(result.amount, result.recipient);
    }

    function transferToNear(uint256 _amount, string calldata _nearReceiverAccountId)
    external pausable (PAUSED_XFER_TO_NEAR) {
        _burn(_msgSender(), _amount);
        emit TransferToNearInitiated(_msgSender(), _amount, _nearReceiverAccountId);
    }

    function _decodeBridgeResult(bytes memory data) internal view returns(BridgeResult memory result) {
        Borsh.Data memory borshData = Borsh.from(data);
        uint8 flag = borshData.decodeU8();
        require(flag == 0, "ERR_NOT_WITHDRAW_RESULT");
        result.amount = borshData.decodeU128();
        bytes20 token = borshData.decodeBytes20();
        result.token = address(uint160(token));

        require(result.token == address(this), "Invalid transfer");

        bytes20 recipient = borshData.decodeBytes20();
        result.recipient = address(uint160(recipient));
    }
}
