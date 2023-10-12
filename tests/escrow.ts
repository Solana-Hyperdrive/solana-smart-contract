import * as anchor from "@coral-xyz/anchor";
import { web3, BN } from '@coral-xyz/anchor'
import { Program } from "@coral-xyz/anchor";
import { AnchorProvider, Wallet } from "@project-serum/anchor";
import { Escrow } from "../target/types/escrow";
import { Connectivity, MAX_FEE_RATE } from "./connectivity";

const log = console.log;

async function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

describe("default_impl", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env()
  const connection = provider.connection;
  const program = anchor.workspace.Escrow as Program<Escrow>;
  const sendKp1 = web3.Keypair.generate()
  const revKp1 = web3.Keypair.generate()
  const revKp2 = web3.Keypair.generate()
  const receiver1 = revKp1.publicKey
  const receiver2 = revKp2.publicKey
  const sender1 = sendKp1.publicKey
  const feeRecever = web3.Keypair.generate().publicKey
  const web3Config = { programId: program.programId.toBase58(), rpcEndpoint: provider.connection.rpcEndpoint }
  const userConn = new Connectivity(provider.wallet, web3Config);

  // const holdingTime = 6 * 60; // 6 min
  // const holdingTime = 2 * 60; // 2 min
  const holdingTime = 2; // 2 sec
  const feeRate = 0.002 * MAX_FEE_RATE //(0.2%)
  // admin SetUpdate
  it("init main State", async () => {
    const mainState = userConn.mainState;
    const info = await connection.getAccountInfo(mainState)
    if (info) return;
    const signature = await program.methods.initMainState(feeRecever, new BN(feeRate), new BN(holdingTime)).accounts({
      systemProgram: web3.SystemProgram.programId,
      mainState,
      owner: provider.publicKey,
    }).rpc();

    log({ signature })
  })

  it("udpate main state", async () => {
    const mainState = userConn.mainState;
    const signature = await program.methods.updateMainState(feeRecever, new BN(feeRate), new BN(holdingTime)).accounts({
      mainState,
      owner: provider.publicKey,
    }).rpc();

    log({ signature })
  })
  //
  // it("udpate main state owner", async () => {
  //   const mainState = userConn.mainState;
  //   const signature = await program.methods.updateMainStateOwner(feeRecever).accounts({
  //     mainState,
  //     owner: provider.publicKey,
  //   }).rpc();
  //
  //   log({ signature })
  // })
  //
  // it("udpate main state (unauthorised owner)", async () => {
  //   const mainState = userConn.mainState;
  //   try {
  //     const signature = await program.methods.updateMainState(feeRecever, new BN(feeRate), new BN(holdingTime)).accounts({
  //       mainState,
  //       owner: provider.publicKey,
  //     }).rpc();
  //     log({ signature })
  //     throw "need to fail"
  //   } catch (error) {
  //     // log({ error })
  //   }
  // })
  //
  // it("fetch mainState: ", async () => {
  //   const state = await program.account.mainState.fetch(userConn.mainState);
  //   const _state = JSON.parse(JSON.stringify(state))
  //   _state.feeRate = state.feeRate.toNumber()
  //   _state.holdingTime = state.holdingTime.toNumber()
  //   log({ _state })
  // })


  let vaultAccount: string = null;
  let token: string = null

  it("create vault main", async () => {
    const user = provider.publicKey
    const { ixs, mintKp } = await userConn.baseSpl.__getCreateTokenInstructions({ mintAuthority: user, decimal: 4, mintingInfo: { tokenAmount: 100 } })
    const tx = new web3.Transaction().add(...ixs);
    const mintTokenTxSign = await provider.sendAndConfirm(tx, [mintKp])
    log({ mintTokenTxSign })

    token = mintKp.publicKey.toBase58()
    const amount = 10;
    const receiver = receiver1
    // const receiver = user

    const res = await userConn.createVault({ token, receiver, amount })
    vaultAccount = res.Ok.info.vaultAccount

    log({ sign: res.Ok.signature })
  })


  // it("revert payment", async () => {
  //   await sleep(3_000)
  //   const res = await userConn.revertPayment(vaultAccount)
  //   log({ log: res?.Err?.logs })
  //   log({ sing: res?.Ok?.signature })
  // })

  it("it redeem payment", async () => {
    const userConn = new Connectivity(new Wallet(revKp1), web3Config)

    await connection.requestAirdrop(userConn.provider.publicKey, web3.LAMPORTS_PER_SOL)
    await sleep(3_000)

    const res = await userConn.redeemPayment(vaultAccount);
    log({ res })
    log({ log: res?.Err?.logs })

  })

});
