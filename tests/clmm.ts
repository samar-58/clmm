import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Clmm } from "../target/types/clmm";
import { assert } from "chai";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import {
  createMint,
  TOKEN_PROGRAM_ID,
  mintTo,
  getAccount,
  createAssociatedTokenAccount,
} from "@solana/spl-token";

describe("clmm", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.clmm as Program<Clmm>;
  const provider = program.provider as anchor.AnchorProvider;
  const wallet = provider.wallet as anchor.Wallet;

  const TICK_SPACING = 60;
  const INITIAL_SQRT_PRICE = new anchor.BN("79228162514264337593543950336"); // sqrt(1) * 2^96
  const TICKS_PER_ARRAY = 10;

  let tokenMint0: PublicKey;
  let tokenMint1: PublicKey;
  let poolPda: PublicKey;
  let tokenVault0Keypair: Keypair;
  let tokenVault1Keypair: Keypair;
  let userTokenAccount0: PublicKey;
  let userTokenAccount1: PublicKey;

  const LOWER_TICK = -600;
  const UPPER_TICK = 60;
  const LIQUIDITY_AMOUNT = new anchor.BN("100000");

  function i32ToLeBytes(value: number): Buffer {
    const buf = Buffer.allocUnsafe(4);
    buf.writeInt32LE(value, 0);
    return buf;
  }

  function getTickArrayStartIndex(tick: number, tickSpacing: number): number {
    const arrayIdx = Math.floor(Math.floor(tick / tickSpacing) / TICKS_PER_ARRAY);
    return arrayIdx * TICKS_PER_ARRAY * tickSpacing;
  }

  before(async () => {
    const mintA = await createMint(provider.connection, wallet.payer, wallet.publicKey, null, 6);
    const mintB = await createMint(provider.connection, wallet.payer, wallet.publicKey, null, 6);

    // token_0 must be < token_1 (program requires this)
    if (mintA.toBuffer().compare(mintB.toBuffer()) < 0) {
      tokenMint0 = mintA;
      tokenMint1 = mintB;
    } else {
      tokenMint0 = mintB;
      tokenMint1 = mintA;
    }

    [poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), tokenMint0.toBuffer(), tokenMint1.toBuffer(), i32ToLeBytes(TICK_SPACING)],
      program.programId
    );

    tokenVault0Keypair = Keypair.generate();
    tokenVault1Keypair = Keypair.generate();

    userTokenAccount0 = await createAssociatedTokenAccount(provider.connection, wallet.payer, tokenMint0, wallet.publicKey);
    userTokenAccount1 = await createAssociatedTokenAccount(provider.connection, wallet.payer, tokenMint1, wallet.publicKey);

    await mintTo(provider.connection, wallet.payer, tokenMint0, userTokenAccount0, wallet.publicKey, 1_000_000_000);
    await mintTo(provider.connection, wallet.payer, tokenMint1, userTokenAccount1, wallet.publicKey, 1_000_000_000);

    console.log("token_0:", tokenMint0.toBase58());
    console.log("token_1:", tokenMint1.toBase58());
    console.log("pool:", poolPda.toBase58());
  });

  it("initializes the pool", async () => {
    await program.methods
      .initializePool(TICK_SPACING, INITIAL_SQRT_PRICE)
      .accountsStrict({
        signer: wallet.publicKey,
        token0Mint: tokenMint0,
        token1Mint: tokenMint1,
        pool: poolPda,
        token0Vault: tokenVault0Keypair.publicKey,
        token1Vault: tokenVault1Keypair.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([tokenVault0Keypair, tokenVault1Keypair])
      .rpc();

    const pool = await program.account.pool.fetch(poolPda);
    assert.equal(pool.tickSpacing, TICK_SPACING);
    assert.equal(pool.token0.toBase58(), tokenMint0.toBase58());
    assert.equal(pool.token1.toBase58(), tokenMint1.toBase58());
    assert.equal(pool.globalLiquidity.toString(), "0");

    console.log("pool initialized, tick:", pool.currentTick);
  });

  it("opens a position and adds liquidity", async () => {
    const lowerStart = getTickArrayStartIndex(LOWER_TICK, TICK_SPACING);
    const upperStart = getTickArrayStartIndex(UPPER_TICK, TICK_SPACING);

    const [lowerTickArrayPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("tick_array"), poolPda.toBuffer(), i32ToLeBytes(lowerStart)],
      program.programId
    );
    const [upperTickArrayPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("tick_array"), poolPda.toBuffer(), i32ToLeBytes(upperStart)],
      program.programId
    );
    const [positionPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("position"),
        poolPda.toBuffer(),
        wallet.publicKey.toBuffer(),
        i32ToLeBytes(LOWER_TICK),
        i32ToLeBytes(UPPER_TICK),
      ],
      program.programId
    );

    const before0 = await getAccount(provider.connection, userTokenAccount0);
    const before1 = await getAccount(provider.connection, userTokenAccount1);

    await program.methods
      .openPosition(UPPER_TICK, LOWER_TICK, lowerStart, upperStart, LIQUIDITY_AMOUNT)
      .accountsStrict({
        signer: wallet.publicKey,
        pool: poolPda,
        token0: tokenMint0,
        token1: tokenMint1,
        lowerTickArray: lowerTickArrayPda,
        upperTickArray: upperTickArrayPda,
        position: positionPda,
        user0: userTokenAccount0,
        user1: userTokenAccount1,
        poolVault0: tokenVault0Keypair.publicKey,
        poolVault1: tokenVault1Keypair.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    // verify position
    const pos = await program.account.position.fetch(positionPda);
    assert.equal(pos.lowerTick, LOWER_TICK);
    assert.equal(pos.upperTick, UPPER_TICK);
    assert.equal(pos.liquidity.toString(), LIQUIDITY_AMOUNT.toString());

    // verify pool liquidity increased
    const pool = await program.account.pool.fetch(poolPda);
    assert.equal(pool.globalLiquidity.toString(), LIQUIDITY_AMOUNT.toString());

    // verify tokens were transferred
    const after0 = await getAccount(provider.connection, userTokenAccount0);
    const after1 = await getAccount(provider.connection, userTokenAccount1);
    const sent0 = before0.amount - after0.amount;
    const sent1 = before1.amount - after1.amount;
    console.log("deposited token_0:", sent0.toString(), "token_1:", sent1.toString());
    assert.isTrue(sent0 > 0n || sent1 > 0n, "should deposit at least one token");
  });

  it("swaps token_0 for token_1 (a_to_b)", async () => {
    const pool = await program.account.pool.fetch(poolPda);
    const tickArrayStart = getTickArrayStartIndex(pool.currentTick, TICK_SPACING);

    const [tickArrayPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("tick_array"), poolPda.toBuffer(), i32ToLeBytes(tickArrayStart)],
      program.programId
    );

    const before0 = await getAccount(provider.connection, userTokenAccount0);
    const before1 = await getAccount(provider.connection, userTokenAccount1);

    const swapAmount = new anchor.BN(100);
    const minOut = new anchor.BN(0);

    await program.methods
      .swap(swapAmount, true, minOut)
      .accountsStrict({
        signer: wallet.publicKey,
        pool: poolPda,
        tickArray: tickArrayPda,
        user0: userTokenAccount0,
        user1: userTokenAccount1,
        tokenVault0: tokenVault0Keypair.publicKey,
        tokenVault1: tokenVault1Keypair.publicKey,
        token0: tokenMint0,
        token1: tokenMint1,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const after0 = await getAccount(provider.connection, userTokenAccount0);
    const after1 = await getAccount(provider.connection, userTokenAccount1);

    const spent = before0.amount - after0.amount;
    const received = after1.amount - before1.amount;

    console.log("swap a_to_b: spent token_0:", spent.toString(), "received token_1:", received.toString());

    assert.isTrue(spent > 0n, "should spend token_0");
    assert.isTrue(received > 0n, "should receive token_1");

    // verify pool price moved down
    const poolAfter = await program.account.pool.fetch(poolPda);
    assert.isTrue(
      BigInt(poolAfter.sqrtPriceX96.toString()) <= BigInt(pool.sqrtPriceX96.toString()),
      "price should decrease for a_to_b swap"
    );
    console.log("tick moved from", pool.currentTick, "to", poolAfter.currentTick);
  });
});
