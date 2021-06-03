const { BN, constants, expectEvent, expectRevert } = require('@openzeppelin/test-helpers');
const { expect } = require('chai');

const { serialize } = require('rainbow-bridge-lib/rainbow/borsh.js');
const { borshifyOutcomeProof } = require('rainbow-bridge-lib/rainbow/borshify-proof.js');

const { hexToBytes } = web3.utils;

const {ethers} = require('ethers')

const NearProverMock = artifacts.require('NearProverMock');
const eNear = artifacts.require('eNearMock');

const proof_template = require('./proof_template.json');

const SCHEMA = {
  'MigrateNearToEthereum': {
    kind: 'struct', fields: [
      ['flag', 'u8'],
      ['amount', 'u128'],
      ['recipient', [20]],
    ]
  }
};

const UNPAUSED_ALL = 0
const PAUSED_FINALISE_FROM_NEAR = 1 << 0
const PAUSED_XFER_TO_NEAR = 1 << 1

contract('eNear bridging', function ([deployer, eNearAdmin, alice, bob]) {

  const name = 'eNear';
  const symbol = 'eNear';

  const ONE_HUNDRED_TOKENS = new BN('100').mul(new BN('10').pow(new BN('24')))

  beforeEach(async () => {
    this.proverMock = await NearProverMock.new({from: deployer})

    this.token = await eNear.new(
      name,
      symbol,
      Buffer.from('eNearBridge', 'utf-8'),
      this.proverMock.address,
      '0',
      eNearAdmin,
      0,
      {from: deployer}
    )
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
          amount: ONE_HUNDRED_TOKENS,
          accountId: 'vince.near'
        }
      )
    })
  })

  describe('finaliseNearToEthTransfer()', () => {
    it('Mints eNear after bridging Near', async () => {
      let proof = JSON.parse(JSON.stringify(proof_template));

      const amount = ethers.utils.parseUnits('1', '24');
      proof.outcome_proof.outcome.status.SuccessValue = serialize(SCHEMA, 'MigrateNearToEthereum', {
        flag: 0,
        amount: amount.toString(),
        recipient: hexToBytes(bob),
      }).toString('base64');

      const receiverBalance = await token.balanceOf(bob);

      await this.token.finaliseNearToEthTransfer(borshifyOutcomeProof(proof), 1099);

      const newReceiverBalance = await this.token.balanceOf(bob);
      expect(newReceiverBalance.sub(receiverBalance).toString()).to.be.equal(amount.toString());
    });

    it('Reverts when reusing proof event', async () => {
      let proof = JSON.parse(JSON.stringify(proof_template));

      const amount = ethers.utils.parseUnits('1', '24');
      proof.outcome_proof.outcome.status.SuccessValue = serialize(SCHEMA, 'MigrateNearToEthereum', {
        flag: 0,
        amount: amount.toString(),
        recipient: hexToBytes(bob),
      }).toString('base64');

      await this.token.finaliseNearToEthTransfer(borshifyOutcomeProof(proof), 1099);

      await expectRevert(
        this.token.finaliseNearToEthTransfer(borshifyOutcomeProof(proof), 1099),
        "The burn event proof cannot be reused"
      )
    })

    it('Reverts when event comes from the wrong executor', async () => {
      let proof = JSON.parse(JSON.stringify(proof_template));
      proof.outcome_proof.outcome.executor_id = 'eNearBridgeInvalid'

      const amount = ethers.utils.parseUnits('1', '24');
      proof.outcome_proof.outcome.status.SuccessValue = serialize(SCHEMA, 'MigrateNearToEthereum', {
        flag: 0,
        amount: amount.toString(),
        recipient: hexToBytes(bob),
      }).toString('base64');

      await expectRevert(
        this.token.finaliseNearToEthTransfer(borshifyOutcomeProof(proof), 1099),
        "Can only unlock tokens from the linked proof producer on Near blockchain"
      )
    })

    it('Reverts if flag is not zero', async () => {
      let proof = JSON.parse(JSON.stringify(proof_template));

      const amount = ethers.utils.parseUnits('1', '24');
      proof.outcome_proof.outcome.status.SuccessValue = serialize(SCHEMA, 'MigrateNearToEthereum', {
        flag: 3,
        amount: amount.toString(),
        recipient: hexToBytes(bob),
      }).toString('base64');

      await expectRevert(
        this.token.finaliseNearToEthTransfer(borshifyOutcomeProof(proof), 1099),
        "ERR_NOT_WITHDRAW_RESULT"
      )
    })
  })
})
