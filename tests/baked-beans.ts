import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BakedBeans } from "../target/types/baked_beans";
import { IDL } from "../target/types/baked_beans";
import {
  LAMPORTS_PER_SOL,
  PublicKey,
  Keypair,
  Transaction,
  SystemProgram,
  sendAndConfirmTransaction,
  SYSVAR_RENT_PUBKEY,
  clusterApiUrl,
  Connection
} from "@solana/web3.js";

const GLOBAL_STATE_SEED = "GLOBAL_STATE_SEED";
const VAULT_SEED = "VAULT_SEED";
const USER_STATE_SEED = "USER_STATE_SEED";

const delay = (delayInms) => {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve(1);
    }, delayInms);
  });
}

describe("baked-beans", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BakedBeans as Program<BakedBeans>;

  let user = Keypair.generate();
  console.log('user =', user.publicKey.toBase58());
  let admin = Keypair.generate();
  console.log('admin =', admin.publicKey.toBase58());

  it("Is initialized!", async () => {
    // Add your test here.
    await airdropSol(provider, user.publicKey, 10000000000); // 10 sol
    await airdropSol(provider, admin.publicKey, 10000000000);
    console.log(program.programId.toBase58());
    const [globalStateKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_STATE_SEED)],
      program.programId
    );
    console.log(globalStateKey.toBase58());
    const [vaultKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(VAULT_SEED)],
      program.programId
    );
    console.log(vaultKey.toBase58());
    const tx = new Transaction().add(
      await program.methods
        .initialize(admin.publicKey)
        .accounts({
          authority: admin.publicKey,
          globalState: globalStateKey,
          devAccount: admin.publicKey,
          marketingAccount: admin.publicKey,
          ceoAccount: admin.publicKey,
          giveawayAccount: admin.publicKey,
          vault: vaultKey,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY
        })
        .instruction()
    );
    //let simulRes = await provider.simulate(tx, [admin]);
    //console.log('simulRes =', simulRes);
    let txHash = await sendAndConfirmTransaction(provider.connection, tx, [admin]);
    console.log("Your transaction signature", txHash);
  });

  it("buy and bake beans", async () => {
    const [globalStateKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_STATE_SEED)],
      program.programId
    );
    const [vaultKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(VAULT_SEED)],
      program.programId
    );
    const [userStateKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.publicKey.toBuffer()],
      program.programId
    );
    const [adminUserStateKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), admin.publicKey.toBuffer()],
      program.programId
    );
    let userData = await program.account.userState.fetchNullable(userStateKey);
    let instructions = [];
    if (!userData) {
      instructions.push(
        await createUserStateInstruction(
          program,
          user,
          user.publicKey,
          userStateKey
        )
      )
    }
    let referrer = admin.publicKey;
    
    const [refUserStateKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), referrer.toBuffer()],
      program.programId
    );
    
    let referrerUserStateKey = refUserStateKey;
    let refUserData = await program.account.userState.fetchNullable(refUserStateKey);
    if (!refUserData) {
      
      referrerUserStateKey = adminUserStateKey;
      referrer = admin.publicKey;

      let adminUserData = await program.account.userState.fetchNullable(adminUserStateKey);
      if (!adminUserData) {
        instructions.push(
          await createUserStateInstruction(
            program,
            user,
            admin.publicKey,
            adminUserStateKey
          )
        )
      }
    }

    let globalData = await program.account.globalState.fetch(globalStateKey);
    const tx = new Transaction();
    if (instructions.length > 0) tx.add(...instructions);
    tx.add(
      await program.methods
        .buyBeans(referrer, new anchor.BN(5).mul(new anchor.BN(LAMPORTS_PER_SOL)))
        .accounts({
          user: user.publicKey,
          globalState: globalStateKey,
          devAccount: globalData.devAccount,
          ceoAccount: globalData.ceoAccount,
          marketingAccount: globalData.marketingAccount,
          vault: vaultKey,
          userState: userStateKey,
          refUserState: referrerUserStateKey,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY
        })
        .instruction()
    );
    tx.add(
      await program.methods
        .bakeBeans(0)
        .accounts({
          user: user.publicKey,
          globalState: globalStateKey,
          vault: vaultKey,
          userState: userStateKey,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY
        })
        .instruction()
    );
    // let simulRes = await provider.simulate(tx, [user]);
    // console.log('simulRes =', simulRes);
    
    let txHash = await sendAndConfirmTransaction(provider.connection, tx, [user]);
    console.log("Your transaction signature", txHash);
    let solBal = await provider.connection.getBalance(user.publicKey);
    console.log(solBal);

    let userStateData = await program.account.userState.fetch(userStateKey);
    console.log("userStateData.beans", userStateData.beans.toNumber());
  })

  it("eat beans", async () => {
    await delay(2000);
    const [globalStateKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_STATE_SEED)],
      program.programId
    );
    const [vaultKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(VAULT_SEED)],
      program.programId
    );
    const [userStateKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.publicKey.toBuffer()],
      program.programId
    );
    let globalData = await program.account.globalState.fetch(globalStateKey);
    const tx = new Transaction().add(
      await program.methods
        .eatBeans()
        .accounts({
          user: user.publicKey,
          globalState: globalStateKey,
          vault: vaultKey,
          
          devAccount: admin.publicKey,
          marketingAccount: admin.publicKey,
          ceoAccount: admin.publicKey,
          giveawayAccount: admin.publicKey,

          userState: userStateKey,
          systemProgram: SystemProgram.programId
        })
        .instruction()
    );
    // let simulRes = await provider.simulate(tx, [user]);
    // console.log('simulRes =', simulRes);
    
    let txHash = await sendAndConfirmTransaction(provider.connection, tx, [user]);
    console.log("Your transaction signature", txHash);
    
    let solBal = await provider.connection.getBalance(user.publicKey);
    console.log(solBal);
  })
});

export const airdropSol = async (
  provider: anchor.Provider,
  target: anchor.web3.PublicKey,
  lamps: number
): Promise<string> => {
  const sig: string = await provider.connection.requestAirdrop(target, lamps);
  await provider.connection.confirmTransaction(sig);
  return sig;
};

export const createUserStateInstruction = async (
  program: any,
  payer: Keypair,
  userKey: PublicKey,
  userStateKey: PublicKey
) => {
  return await program.methods
    .initUserState(userKey)
    .accounts({
      payer: payer.publicKey,
      userState: userStateKey,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .instruction();
};