const { ethers } = require("hardhat");
import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { sortBy } from "lodash";

class Utils {
  static getAddresses = (signers: SignerWithAddress[]) =>
    signers.map(({ address }) => address);

  static getRandomID = () => {
    return ethers.utils.id(Math.floor(Math.random() * 1e10).toString());
  };

  static getWeightedSignaturesProof = async (
    data: string,
    operators: SignerWithAddress[],
    weights: number[],
    threshold: number,
    signers: SignerWithAddress[]
  ) => {
    const hash = ethers.utils.arrayify(ethers.utils.keccak256(data));
    const signatures = await Promise.all(
      sortBy(signers, (wallet) => wallet.address.toLowerCase()).map((wallet) =>
        wallet.signMessage(hash)
      )
    );
    return ethers.utils.defaultAbiCoder.encode(
      ["address[]", "uint256[]", "uint256", "bytes[]"],
      [Utils.getAddresses(operators), weights, threshold, signatures]
    );
  };

  static getTransferWeightedOperatorshipCommand = async (
    newOperators: string[],
    newWeights: number[],
    threshold: number
  ) => {
    return ethers.utils.defaultAbiCoder.encode(
      ["address[]", "uint256[]", "uint256"],
      [
        sortBy(newOperators, (address) => address.toLowerCase()),
        newWeights,
        threshold,
      ]
    );
  };

  static getApproveContractCall = async (
    sourceChain: string,
    source: string,
    destination: string,
    payloadHash: string,
    sourceTxHash: string,
    sourceEventIndex: number
  ) => {
    return ethers.utils.defaultAbiCoder.encode(
      ["string", "string", "string", "bytes32", "bytes32", "uint256"],
      [
        sourceChain,
        source,
        destination,
        payloadHash,
        sourceTxHash,
        sourceEventIndex,
      ]
    );
  };

  static buildCommandBatch = async (
    chainId: number,
    commandIDs: string[],
    commandNames: string[],
    commands: string[]
  ) => {
    return ethers.utils.arrayify(
      ethers.utils.defaultAbiCoder.encode(
        ["uint256", "bytes32[]", "string[]", "bytes[]"],
        [chainId, commandIDs, commandNames, commands]
      )
    );
  };

  static getSignedWeightedExecuteInput = async (
    data: string,
    operators: SignerWithAddress[],
    weights: number[],
    threshold: number,
    signers: SignerWithAddress[]
  ) => {
    return ethers.utils.defaultAbiCoder.encode(
      ["bytes", "bytes"],
      [
        data,
        await Utils.getWeightedSignaturesProof(
          data,
          operators,
          weights,
          threshold,
          signers
        ),
      ]
    );
  };
}

export default Utils;
