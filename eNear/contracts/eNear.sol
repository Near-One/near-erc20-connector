// SPDX-License-Identifier: MIT

pragma solidity 0.6.12;

import "rainbow-bridge/contracts/eth/nearprover/contracts/ProofDecoder.sol";
import "rainbow-bridge/contracts/eth/nearbridge/contracts/Borsh.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { Bridge, INearProver } from "./Bridge.sol";
import { AdminControlled } from "./AdminControlled.sol";

contract eNear is ERC20("eNear","eNear"), Bridge, AdminControlled {

    uint constant UNPAUSED_ALL = 0;
    uint constant PAUSED_FINALISE_FROM_NEAR = 1 << 0;
    uint constant PAUSED_XFER_TO_NEAR = 1 << 1;

    string nameOverride;
    string symbolOverride;

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

    function init(
        string memory _tokenName,
        string memory _tokenSymbol,
        bytes memory _nearTokenFactory,
        INearProver _prover,
        address _admin,
        uint _pausedFlags
    ) public {
        nameOverride = _tokenName;
        symbolOverride = _tokenSymbol;

        require(_nearTokenFactory.length > 0, "Invalid Near Token Factory address");
        require(address(_prover) != address(0), "Invalid Near prover address");

        nearTokenFactory_ = _nearTokenFactory;
        prover_ = _prover;

        admin = _admin;

        // Add the possibility to set pause flags on the initialization
        paused = _pausedFlags;
    }

    function name() public view override returns (string memory) {
        return nameOverride;
    }

    function symbol() public view override returns (string memory) {
        return symbolOverride;
    }

    function decimals() public view override returns (uint8) {
        // set decimals to 24 to mirror yocto Near
        return 24;
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
