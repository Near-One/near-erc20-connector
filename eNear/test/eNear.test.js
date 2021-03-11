const { BN, constants, expectEvent, expectRevert } = require('@openzeppelin/test-helpers');
const { expect } = require('chai');
const { ZERO_ADDRESS } = constants;

const {
  shouldBehaveLikeERC20,
  shouldBehaveLikeERC20Transfer,
  shouldBehaveLikeERC20Approve,
} = require('./ERC20.behavior');

const eNear = artifacts.require('eNear');
const TransparentUpgradeableProxy = artifacts.require('TransparentUpgradeableProxyMock');

contract('eNear bridging', function ([deployer, ...otherAccounts]) {
  beforeEach(async () => {

  })
})
