// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.24;

import {AuroraSdk, NEAR, PromiseCreateArgs, PromiseWithCallback, Utils} from "@auroraisnear/aurora-sdk/aurora-sdk/AuroraSdk.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import "./IEvmErc20.sol";

contract NearBridge {
    using AuroraSdk for NEAR;
    using AuroraSdk for PromiseCreateArgs;
    using AuroraSdk for PromiseWithCallback;

    NEAR private near;
    string private nearBridgeAccountId;

    uint64 constant MIGRATE_TO_ETHEREUM_GAS = 150_000_000_000_000;

    event InitBridgeToEthereum(address indexed sender, address indexed recipient, uint128 amount);

    constructor(address wnear, string memory _nearBridgeAccountId) {
        near = AuroraSdk.initNear(IERC20(wnear));
        nearBridgeAccountId = _nearBridgeAccountId;
    }

    function bridgeToEthereum(address recipient, uint128 amount) external {
        string memory recipientStr = Utils.bytesToHex(abi.encodePacked(recipient));

        PromiseCreateArgs memory callMigrateToEthereum = near.call(
            nearBridgeAccountId,
            "migrate_to_ethereum",
            bytes(string.concat('{"eth_recipient": "', recipientStr, '" }')),
            amount,
            MIGRATE_TO_ETHEREUM_GAS
        );
        
        callMigrateToEthereum.transact();

        emit InitBridgeToEthereum(msg.sender, recipient, amount);
    }
}