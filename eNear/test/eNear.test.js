const { BN, constants, expectEvent, expectRevert } = require('@openzeppelin/test-helpers');
const { expect } = require('chai');
const { ZERO_ADDRESS } = constants;

const eNear = artifacts.require('eNearMock');
const TransparentUpgradeableProxy = artifacts.require('TransparentUpgradeableProxyMock');

const eNearABI = require('../artifacts/contracts/eNear.sol/eNear.json').abi

contract('eNear bridging', function ([deployer, proxyAdmin, prover, eNearAdmin, alice, ...otherAccounts]) {

  const name = 'eNear';
  const symbol = 'eNear';

  const ONE_HUNDRED_TOKENS = new BN('100').mul(new BN('10').pow(new BN('24')))

  beforeEach(async () => {
    this.logic = await eNear.new()

    // deploys the proxy and calls init on the implementation
    this.proxy = await TransparentUpgradeableProxy.new(
      this.logic.address,
      proxyAdmin,
      await new web3.eth.Contract(eNearABI).methods.init(
        name,
        symbol,
        Buffer.from('factory', 'utf-8'),
        prover,
        eNearAdmin,
        0
      ).encodeABI(),
      {from: deployer}
    )

    this.token = await eNear.at(this.proxy.address)
  })

  describe('transferToNear()', () => {
    it('Burns eNear when transferring to near', async () => {
      // check supply zero and balance zero
      expect(await this.token.totalSupply()).to.be.bignumber.equal('0')
      expect(await this.token.balanceOf(alice)).to.be.bignumber.equal('0')

      // mint some tokens to account bridging
      await this.token.mintTo(alice, ONE_HUNDRED_TOKENS)

      // check supply and balance
      expect(await this.token.totalSupply()).to.be.bignumber.equal(ONE_HUNDRED_TOKENS)
      expect(await this.token.balanceOf(alice)).to.be.bignumber.equal(ONE_HUNDRED_TOKENS)

      // call xfer to near
      const {receipt} = await this.token.transferToNear(
        ONE_HUNDRED_TOKENS,
        'vince.near',
        {from: alice}
      )

      // check supply zero and balance zero
      expect(await this.token.totalSupply()).to.be.bignumber.equal('0')
      expect(await this.token.balanceOf(alice)).to.be.bignumber.equal('0')

      // check event emitted
      await expectEvent(
        receipt, 'TransferToNearInitiated', {
          sender: alice,
          amount: ONE_HUNDRED_TOKENS,
          accountId: 'vince.near'
        }
      )
    })
  })
})
