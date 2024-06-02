import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ArmoryStaking } from "../target/types/armory_staking";

describe("Armory-Staking", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ArmoryStaking as Program<ArmoryStaking>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
