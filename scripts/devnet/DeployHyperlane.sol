// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.30;

import {console} from "forge-std/console.sol";
import {Script} from "forge-std/Script.sol";

import {Mailbox} from "@hyperlane/Mailbox.sol";
import {HypNative} from "@hyperlane/token/HypNative.sol";
import {MerkleTreeHook} from "@hyperlane/hooks/MerkleTreeHook.sol";
import {PausableHook} from "@hyperlane/hooks/PausableHook.sol";
import {TrustedRelayerIsm} from "@hyperlane/isms/TrustedRelayerIsm.sol";

import {ProxyAdmin} from "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";
import {TransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

contract DeployHyperlane is Script {
    address AUTHORITY = 0xFC28736049E1ea4A315bFc4CfC6e09240250dfdf;

    function run() public {
        vm.startBroadcast();

        // Deploy Mailbox

        Mailbox mailboxImplementation = new Mailbox(uint32(block.chainid));
        console.log("Mailbox Implementation: %s", address(mailboxImplementation));

        ProxyAdmin mailboxAdmin = new ProxyAdmin();
        console.log("Mailbox Admin: %s", address(mailboxAdmin));
        mailboxAdmin.transferOwnership(AUTHORITY);

        TransparentUpgradeableProxy mailboxProxy =
            new TransparentUpgradeableProxy(address(mailboxImplementation), address(mailboxAdmin), "");
        Mailbox mailbox = Mailbox(address(mailboxProxy));
        console.log("Mailbox: %s", address(mailbox));

        // Deploy ISM

        TrustedRelayerIsm ism = new TrustedRelayerIsm(address(mailbox), msg.sender);
        console.log("ISM: %s", address(ism));

        // Deploy Hooks

        PausableHook defaultHook = new PausableHook();
        console.log("Default Hook: %s", address(defaultHook));

        MerkleTreeHook requiredHook = new MerkleTreeHook(address(mailbox));
        console.log("Required Hook: %s", address(requiredHook));

        // Initialize Mailbox
        mailbox.initialize(msg.sender, address(ism), address(defaultHook), address(requiredHook));

        // Deploy HypNative ($NOBLE)
        HypNative nativeImplementation = new HypNative(1, address(mailbox));
        console.log("HypNative Implementation: %s", address(nativeImplementation));

        ProxyAdmin nativeAdmin = new ProxyAdmin();
        console.log("HypNative Admin: %s", address(nativeAdmin));
        nativeAdmin.transferOwnership(AUTHORITY);

        TransparentUpgradeableProxy nativeProxy = new TransparentUpgradeableProxy(
            address(nativeImplementation),
            address(nativeAdmin),
            abi.encodeWithSelector(HypNative.initialize.selector, address(0), address(0), msg.sender)
        );
        HypNative native = HypNative(payable(address(nativeProxy)));
        console.log("HypNative: %s", address(native));

        // Configure HypNative ($NOBLE)
        native.enrollRemoteRouter(1146440517, 0x726f757465725f61707000000000000000000000000000020000000000000000);

        // We choose to not deploy HypFiatToken ($USDC) in this script, as it
        // requires $USDC to be deployed first, which is done via the Circle
        // scripts.

        vm.stopBroadcast();
    }
}
