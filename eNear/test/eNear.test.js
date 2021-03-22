const { BN, constants, expectEvent, expectRevert } = require('@openzeppelin/test-helpers');
const { expect } = require('chai');
const { ZERO_ADDRESS } = constants;

const { serialize } = require('rainbow-bridge-lib/rainbow/borsh.js');
const { borshifyOutcomeProof } = require('rainbow-bridge-lib/rainbow/borshify-proof.js');

const { toWei, fromWei, hexToBytes } = web3.utils;

const {ethers} = require('ethers')

const NearProverMock = artifacts.require('NearProverMock');
const eNear = artifacts.require('eNearMock');
const TransparentUpgradeableProxy = artifacts.require('TransparentUpgradeableProxyMock');

const eNearABI = require('../artifacts/contracts/eNear.sol/eNear.json').abi

const SCHEMA = {
  'Unlock': {
    kind: 'struct', fields: [
      ['flag', 'u8'],
      ['amount', 'u128'],
      ['token', [20]],
      ['recipient', [20]],
    ]
  }
};

const UNPAUSED_ALL = 0
const PAUSED_FINALISE_FROM_NEAR = 1 << 0
const PAUSED_XFER_TO_NEAR = 1 << 1

contract('eNear bridging', function ([deployer, proxyAdmin, prover, eNearAdmin, alice, bob, ...otherAccounts]) {

  const name = 'eNear';
  const symbol = 'eNear';

  const ONE_HUNDRED_TOKENS = new BN('100').mul(new BN('10').pow(new BN('24')))

  beforeEach(async () => {
    this.proverMock = await NearProverMock.new()

    this.logic = await eNear.new()

    // deploys the proxy and calls init on the implementation
    this.proxy = await TransparentUpgradeableProxy.new(
      this.logic.address,
      proxyAdmin,
      await new web3.eth.Contract(eNearABI).methods.init(
        name,
        symbol,
        Buffer.from('eNearBridge', 'utf-8'),
        this.proverMock.address,
        eNearAdmin,
        0
      ).encodeABI(),
      {from: deployer}
    )

    this.token = await eNear.at(this.proxy.address)
  })

  describe('init()', () => {
    it('Reverts when version already initialised', async () => {
      await expectRevert(
        this.token.init(
          name,
          symbol,
          Buffer.from('eNearBridge', 'utf-8'),
          this.proverMock.address,
          eNearAdmin,
          0
        ),
        "Can only call init() once per version"
      )
    })
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

  describe('finaliseNearToEthTransfer()', () => {
    it('Mints eNear after bridging Near', async () => {
      let proof = require('./proof_template.json');

      const amount = ethers.utils.parseUnits('1', '24');
      proof.outcome_proof.outcome.status.SuccessValue = serialize(SCHEMA, 'Unlock', {
        flag: 0,
        amount: amount.toString(),
        token: hexToBytes(this.token.address),
        recipient: hexToBytes(bob),
      }).toString('base64');

      const receiverBalance = await token.balanceOf(bob);

      await this.token.finaliseNearToEthTransfer(borshifyOutcomeProof(proof), 1099);

      const newReceiverBalance = await this.token.balanceOf(bob);
      expect(newReceiverBalance.sub(receiverBalance).toString()).to.be.equal(amount);
    });
  })
})
