import { Program, web3, BN, AnchorProvider } from '@project-serum/anchor'
import { Wallet as AWallet } from '@project-serum/anchor/dist/browser/src/provider'
import { IDL, Escrow } from "../target/types/escrow";
import { BaseSpl } from './baseSpl';
import { utf8 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';
import { getAssociatedTokenAddressSync, MintLayout, TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { ASSOCIATED_PROGRAM_ID } from '@project-serum/anchor/dist/cjs/utils/token';

const log = console.log;
export const MAX_FEE_RATE = 1000_000;
const Seeds = {
  mainState: utf8.encode("main"),
  vault: utf8.encode('vault'),
}
const tokenProgram = TOKEN_PROGRAM_ID
const systemProgram = web3.SystemProgram.programId
const associatedTokenProgram = ASSOCIATED_PROGRAM_ID

export function getNonDecimalVaule(value: number, decimal): number {
  return Math.trunc((10 ** decimal) * value);
}
export function getFloatingVaule(value: number, decimal): number {
  return value / (10 ** decimal)
}

export type Result<T, E> = {
  Ok?: T;
  Err?: E;
};
export type TxPassType<Info> = { signature: string, info?: Info };

export class Connectivity {
  programId: web3.PublicKey;
  provider: AnchorProvider;
  txis: web3.TransactionInstruction[] = [];
  program: Program<Escrow>;
  mainState: web3.PublicKey;
  connection: web3.Connection;
  baseSpl: BaseSpl

  constructor(wallet: AWallet, web3Config: { rpcEndpoint: string, programId: string }) {
    if (!wallet) return;
    const connection = new web3.Connection(web3Config.rpcEndpoint, { commitment: 'confirmed' });
    const provider = new AnchorProvider(
      connection, wallet,
      { commitment: 'confirmed' }
    )

    this.provider = provider;
    this.connection = provider.connection
    this.programId = new web3.PublicKey(web3Config.programId)
    this.program = new Program(IDL, this.programId, this.provider);
    this.mainState = web3.PublicKey.findProgramAddressSync(
      [Seeds.mainState],
      this.programId
    )[0];
    this.baseSpl = new BaseSpl(this.connection)
  }

  ixCallBack = (ixs?: web3.TransactionInstruction[]) => {
    if (ixs) {
      this.txis.push(...ixs)
    }
  }
  reInit() {
    this.txis = []
  }

  getVaultAccount(sender: web3.PublicKey, id: number) {
    return web3.PublicKey.findProgramAddressSync([
      Seeds.vault,
      sender.toBytes(),
      new BN(id).toBuffer('le', 8),
    ], this.programId)[0]
  }

  async getVaultInfo(vaultAccount: web3.PublicKey | string) {
    if (typeof vaultAccount == 'string') vaultAccount = new web3.PublicKey(vaultAccount)
    try {
      const info = await this.program.account.vaultState.fetch(vaultAccount)
      const parseInfo = JSON.parse(JSON.stringify(info))
      log({ parseInfo })

    } catch (vaultInfoFetchError) {
      log({ vaultInfoFetchError })
      return null;
    }
  }

  // async getVaultInfo(args: { id?: number, sender?: web3.PublicKey | string, vaultAccount?: web3.PublicKey }) {
  //   let {
  //     sender,
  //     id,
  //     vaultAccount,
  //   } = args;
  //
  //   let _vaultAccount = null;
  //   if (vaultAccount) {
  //     _vaultAccount = vaultAccount
  //   } else {
  //     if (!sender || !id) throw "Unable to prase args to get vaultAccount"
  //     if (typeof sender == 'string') sender = new web3.PublicKey(sender)
  //     _vaultAccount = this.getVaultAccount(sender, id);
  //   }
  //
  //   return await this._getVaultInfo(vaultAccount);
  // }

  async createVault(args: {
    amount: number,
    token: web3.PublicKey | string
    receiver: web3.PublicKey | string
  }): Promise<Result<TxPassType<{
    id: number,
    sender: string,
    receiver: string,
    token: string,
    amount: number
    vaultAccount: string
  }>, any>> {
    try {
      this.reInit()
      const sender = this.provider.publicKey;
      if (!sender) "throw wallet not found"
      let { token, amount, receiver } = args;
      if (typeof token == 'string') token = new web3.PublicKey(token)
      if (typeof receiver == 'string') receiver = new web3.PublicKey(receiver)
      const isSol = false;
      const id = Date.now();
      const feeReceiver = (await this.program.account.mainState.fetch(this.mainState)).feeReceiver
      const { ata: feeReceiverAta } = await this.baseSpl.__getOrCreateTokenAccountInstruction({ mint: token, owner: feeReceiver, payer: sender }, this.ixCallBack)

      const vault = this.getVaultAccount(sender, id);
      const vaultAta = getAssociatedTokenAddressSync(token, vault, true);
      const { ata: senderAta } = await this.baseSpl.__getOrCreateTokenAccountInstruction({ owner: sender, mint: token, payer: sender }, this.ixCallBack);
      const tokenDecimal = (MintLayout.decode((await this.connection.getAccountInfo(token)).data)).decimals
      const _amount = getNonDecimalVaule(amount, tokenDecimal);

      const ix = await this.program.methods.createVault({ amount: new BN(_amount), receiver, id: new BN(id) }).accounts({
        token,
        sender,
        senderAta,
        vault,
        vaultAta,
        mainState: this.mainState,
        feeReceiverAta,
        tokenProgram,
        systemProgram,
        associatedTokenProgram,
      }).instruction();
      this.txis.push(ix)
      const tx = new web3.Transaction().add(...this.txis);
      this.txis = []
      const signature = await this.provider.sendAndConfirm(tx);

      return {
        Ok: {
          signature,
          info: {
            amount,
            token: token.toBase58(),
            receiver: receiver.toBase58(),
            sender: sender.toBase58(),
            vaultAccount: vault.toBase58(),
            id
          }
        }
      }
    } catch (error) {
      return { Err: error }
    }
  }

  async revertPayment(
    vaultAccount: web3.PublicKey | string
  ): Promise<Result<TxPassType<any>, any>> {
    try {
      this.reInit();
      const sender = this.provider.publicKey;
      if (!sender) throw "Wallet not found"

      if (typeof vaultAccount == 'string') vaultAccount = new web3.PublicKey(vaultAccount)
      const vaultInfo = await this.program.account.vaultState.fetch(vaultAccount);
      if (sender.toBase58() != vaultInfo.sender.toBase58()) throw "Unknown sender"
      const token = vaultInfo.token
      const { ata: senderAta } = await this.baseSpl.__getOrCreateTokenAccountInstruction({ mint: token, owner: sender },)
      const vaultAta = getAssociatedTokenAddressSync(token, vaultAccount, true);

      const ix = await this.program.methods.revertPayment().accounts({
        systemProgram,
        tokenProgram,
        vaultAta,
        senderAta,
        sender,
        vaultState: vaultAccount,
      }).instruction()
      this.txis.push(ix)

      const tx = new web3.Transaction().add(...this.txis);
      const signature = await this.provider.sendAndConfirm(tx);

      return { Ok: { signature } }
    } catch (error) {
      return { Err: error }
    }
  }

  async redeemPayment(vaultAccount: web3.PublicKey | string): Promise<Result<TxPassType<any>, any>> {
    this.reInit()
    try {
      const receiver = this.provider.publicKey;
      if (!receiver) throw "Wallet not found"
      if (typeof vaultAccount == 'string') vaultAccount = new web3.PublicKey(vaultAccount)
      const vaultInfo = await this.program.account.vaultState.fetch(vaultAccount);
      const token = vaultInfo.token
      const sender = vaultInfo.sender
      if (receiver.toBase58() != vaultInfo.receiver.toBase58()) throw "Unknown Receiver"
      const vaultAta = getAssociatedTokenAddressSync(token, vaultAccount, true);
      const { ata: receiverAta } = await this.baseSpl.__getOrCreateTokenAccountInstruction({
        mint: token, owner: receiver
      }, this.ixCallBack)

      const ix = await this.program.methods.redeemPayment().accounts({
        vaultState: vaultAccount,
        sender,
        vaultAta,
        tokenProgram,
        systemProgram,
        receiver,
        receiverAta,
      }).instruction()
      this.txis.push(ix)

      const tx = new web3.Transaction().add(...this.txis);
      const signature = await this.provider.sendAndConfirm(tx);
      return { Ok: { signature } }
    } catch (error) {
      return { Err: error }
    }
  }
}
