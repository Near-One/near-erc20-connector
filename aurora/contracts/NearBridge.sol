// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.24;

import {AuroraSdk, NEAR, PromiseCreateArgs, Utils} from "@auroraisnear/aurora-sdk/aurora-sdk/AuroraSdk.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

contract NearBridge is UUPSUpgradeable, PausableUpgradeable, OwnableUpgradeable {
    using AuroraSdk for NEAR;
    using AuroraSdk for PromiseCreateArgs;

    NEAR private near;
    string private eNearAccountId;

    uint64 constant MIGRATE_TO_ETHEREUM_GAS = 10_000_000_000_000;
    uint64 constant WITHDRAW_NEAR_GAS = 50_000_000_000_000;
    uint128 constant ONE_YOCTO = 1;

    event InitBridgeToEthereum(address indexed sender, address indexed recipient, uint128 amount);
    
    error AmountIsZero();
    error RecipientIsZeroAddress();

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize(address wnear, string memory _eNearAccountId) external initializer {
        near = AuroraSdk.initNear(IERC20(wnear));
        eNearAccountId = _eNearAccountId;

        __Ownable_init(msg.sender);
        __Pausable_init();
    }

    function bridgeToEthereum(address recipient, uint128 amount) external whenNotPaused {
        if (amount == 0) {
            revert AmountIsZero();
        }

        if (recipient == address(0)) {
            revert RecipientIsZeroAddress();
        }

        string memory recipientStr = Utils.bytesToHex(abi.encodePacked(recipient));

        PromiseCreateArgs memory callMigrateToEthereum = near.call(
            eNearAccountId,
            "migrate_to_ethereum",
            bytes(string.concat('{"eth_recipient": "', recipientStr, '" }')),
            amount,
            MIGRATE_TO_ETHEREUM_GAS
        );

        callMigrateToEthereum.transact();
        
        emit InitBridgeToEthereum(msg.sender, recipient, amount);
    }

    function withdrawFromImplicitNearAccount(
        string calldata receiver,
        string calldata token,
        address recipient,
        uint128 amount
    ) external onlyOwner {
        require(
            near.wNEAR.balanceOf(address(this)) >= ONE_YOCTO,
            "Not enough wNEAR balance"
        );

        bytes memory args = bytes(
            string.concat(
                '{"receiver_id": "',
                receiver,
                '", "amount": "',
                Strings.toString(amount),
                '", "msg": "',
                Utils.bytesToHex(abi.encodePacked(recipient)),
                '"}'
            )
        );

        PromiseCreateArgs memory callWithdraw = _callWithoutTransferWNear(
            near,
            token,
            "ft_transfer_call",
            args,
            ONE_YOCTO,
            WITHDRAW_NEAR_GAS
        );

        callWithdraw.transact();
    }

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}


    function _callWithoutTransferWNear(
        NEAR storage _near,
        string memory targetAccountId,
        string memory method,
        bytes memory args,
        uint128 nearBalance,
        uint64 nearGas
    ) internal view returns (PromiseCreateArgs memory) {
        require(_near.initialized, "Near isn't initialized");
        return
            PromiseCreateArgs(
                targetAccountId,
                method,
                args,
                nearBalance,
                nearGas
            );
    }
}