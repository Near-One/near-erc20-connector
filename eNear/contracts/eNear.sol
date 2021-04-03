// SPDX-License-Identifier: MIT

pragma solidity 0.6.12;

import "rainbow-bridge/contracts/eth/nearprover/contracts/ProofDecoder.sol";
import "rainbow-bridge/contracts/eth/nearbridge/contracts/Borsh.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { Bridge, INearProver } from "./Bridge.sol";
import { AdminControlled } from "./AdminControlled.sol";

contract eNear is ERC20("NEAR","NEAR"), Bridge, AdminControlled {

    uint constant UNPAUSED_ALL = 0;
    uint constant PAUSE_FINALISE_FROM_NEAR = 1 << 0;
    uint constant PAUSE_TRANSFER_TO_NEAR = 1 << 1;

    string nameOverride;
    string symbolOverride;

    uint256 initVersion;

    event TransferToNearInitiated (
        address indexed sender,
        uint256 amount,
        string accountId
    );

    event NearToEthTransferFinalised (
        address indexed sender,
        uint128 amount,
        address indexed recipient
    );

    struct BridgeResult {
        uint128 amount;
        address recipient;
    }

    function init(
        string memory _tokenName,
        string memory _tokenSymbol,
        bytes memory _nearConnector,
        INearProver _prover,
        uint64 _minBlockAcceptanceHeight,
        address _admin,
        uint256 _pausedFlags
    ) public {
        require(version().sub(1) == initVersion, "Can only call init() once per version");

        nameOverride = _tokenName;
        symbolOverride = _tokenSymbol;

        require(_nearConnector.length > 0, "Invalid Near Token Factory address");
        require(address(_prover) != address(0), "Invalid Near prover address");

        nearConnector_ = _nearConnector;
        prover_ = _prover;

        minBlockAcceptanceHeight_ = _minBlockAcceptanceHeight;

        admin = _admin;

        // Add the possibility to set pause flags on the initialization
        paused = _pausedFlags;

        initVersion = version();
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
    external pausable (PAUSE_FINALISE_FROM_NEAR) {
        ProofDecoder.ExecutionStatus memory status = _parseAndConsumeProof(proofData, proofBlockHeight);
        BridgeResult memory result = _decodeBridgeResult(status.successValue);

        _mint(result.recipient, result.amount);

        emit NearToEthTransferFinalised(_msgSender(), result.amount, result.recipient);
    }

    function transferToNear(uint256 _amount, string calldata _nearReceiverAccountId)
    external pausable (PAUSE_TRANSFER_TO_NEAR) {
        _burn(_msgSender(), _amount);
        emit TransferToNearInitiated(_msgSender(), _amount, _nearReceiverAccountId);
    }

    function updateName(string calldata _name) external onlyAdmin {
        nameOverride = _name;
    }

    function updateSymbol(string calldata _symbol) external onlyAdmin {
        symbolOverride = _symbol;
    }

    function version() virtual internal pure returns (uint256) {
        return 1;
    }

    function _decodeBridgeResult(bytes memory data) internal view returns(BridgeResult memory result) {
        Borsh.Data memory borshData = Borsh.from(data);
        uint8 flag = borshData.decodeU8();
        require(flag == 0, "ERR_NOT_WITHDRAW_RESULT");
        result.amount = borshData.decodeU128();
        bytes20 recipient = borshData.decodeBytes20();
        result.recipient = address(uint160(recipient));
    }
}
