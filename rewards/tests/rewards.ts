import * as anchor from "@project-serum/anchor";
import * as splToken from "@solana/spl-token";
import * as mplMd from "@metaplex-foundation/mpl-token-metadata";
import { Program } from "@project-serum/anchor";
import { Rewards } from "../target/types/rewards";

describe("rewards", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  describe("create", () => {
    const program = anchor.workspace.Rewards as Program<Rewards>;
    const wallet = (program.provider as anchor.AnchorProvider).wallet;

    it("create a simple plan", async () => {
      let rewardPlanName = "discount";

      let createRewardPlanParams = {
        name: rewardPlanName,
        threshold: new anchor.BN(1),
        collectionMetadataUri: "collection-url",
        itemMetadataUri: "item-url",
      };

      // holds all the reward plan configuration
      let [rewardPlanConfig, _rewardPlanConfigBump] =
        anchor.web3.PublicKey.findProgramAddressSync(
          [wallet.publicKey.toBuffer(), Buffer.from(rewardPlanName)],
          program.programId
        );
      
      // create collection mint using reward config as the seed
      let [collectionMint, _collectionMintBump] = anchor.web3.PublicKey.findProgramAddressSync(
        [rewardPlanConfig.toBuffer()], program.programId
      );

      let collectionMintAta = await splToken.getAssociatedTokenAddress(collectionMint, wallet.publicKey); 

      // derive metaplex metadata account with seeds it expects
      let [collectionMd, _collectionMdBump] = await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('metadata'), mplMd.PROGRAM_ID.toBuffer(), collectionMint.toBuffer()], mplMd.PROGRAM_ID,
      );

      // derive metaplex master edition account with seeds it expects
      let [collectionMe, _collectionMeBump] = await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('metadata'), mplMd.PROGRAM_ID.toBuffer(), collectionMint.toBuffer(), Buffer.from('edition')], mplMd.PROGRAM_ID,
      );

      const txSig = await program.methods
        .createRewardPlan(createRewardPlanParams)
        .accounts({
          collectionMint: collectionMint,
          collectionMintAta: collectionMintAta,
          collectionMd: collectionMd,
          collectionMe: collectionMe,
          config: rewardPlanConfig,
          admin: wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: splToken.TOKEN_PROGRAM_ID,
          associatedTokenProgram: splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
          metadataProgram: mplMd.PROGRAM_ID,
        })
        .rpc();
      console.log("createRewardPlan txSig: %s", txSig);
    });
  });
});
