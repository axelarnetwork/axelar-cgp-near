import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import anyTest, { TestFn } from "ava";
import { sortBy } from "lodash";
import { NEAR, NearAccount, Worker } from "near-workspaces";
const { ethers } = require("hardhat");

class Utils {
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
      [getAddresses(operators), weights, threshold, signatures]
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
      ["string", "string", "address", "bytes32", "bytes32", "uint256"],
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

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
}>;

const CHAIN_ID = 0;
const ADDRESS_ZERO = "0x0000000000000000000000000000000000000000";
const OLD_KEY_RETENTION = 16;

const threshold = 3;

let wallets: SignerWithAddress[];
let owner: SignerWithAddress;
let operators: SignerWithAddress[];
const previousOperators: SignerWithAddress[][] = [];

const getAddresses = (signers: SignerWithAddress[]) =>
  signers.map(({ address }) => address);

const initAuthContract = async (root: NearAccount, contract: NearAccount) => {
  await contract.deploy(
    "./auth/target/wasm32-unknown-unknown/release/axelar_auth_weighted.wasm"
  );

  const initialOperators: SignerWithAddress[][] = [
    ...previousOperators,
    operators,
  ];

  let operatorSets = initialOperators.map((el) =>
    el.map(({ address }) => address)
  );
  let weightSets = initialOperators.map((el) => Array(el.length).fill(1));
  let operatorThresholds = initialOperators.map(() => threshold);

  const recentOperators = operatorSets.map((operators, i) =>
    ethers.utils.defaultAbiCoder.encode(
      ["address[]", "uint256[]", "uint256"],
      [operators, weightSets[i], operatorThresholds[i]]
    )
  );

  await root.call(contract, "new", {
    recent_operators: recentOperators,
  });
};

test.before(async (t) => {
  wallets = await ethers.getSigners();
  wallets = wallets;

  owner = wallets[0];

  operators = sortBy(wallets.slice(3, 9), (wallet) =>
    wallet.address.toLowerCase()
  );

  let previousOperatorsLimit = OLD_KEY_RETENTION;

  for (let i = 0; i < wallets.length - 3; i++) {
    for (let j = i; j < wallets.length - 3; j++) {
      previousOperators.push(
        sortBy(wallets.slice(i, j + 3), (wallet) =>
          wallet.address.toLowerCase()
        )
      );

      --previousOperatorsLimit;
    }

    if (previousOperatorsLimit <= 0) break;
  }
});

test.beforeEach(async (t) => {
  const worker = await Worker.init();

  const root = worker.rootAccount;

  const john = await root.createSubAccount("john", {
    initialBalance: NEAR.parse("3 N").toJSON(),
  });

  const contract = await root.createSubAccount("axelar_auth_weighted");

  await initAuthContract(root, contract);

  t.context.worker = worker;
  t.context.accounts = { root, contract, john };
});

test.afterEach.always(async (t) => {
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed to stop the Sandbox:", error);
  });
});

// Auth Tests

test("validate the proof from the current operators", async (t) => {
  const { contract } = t.context.accounts;

  const data = "0x123abc123abc";

  const message = ethers.utils.hashMessage(
    ethers.utils.arrayify(ethers.utils.keccak256(data))
  );

  let args = {
    message_hash: message,
    proof: await Utils.getWeightedSignaturesProof(
      data,
      operators,
      operators.map(() => 1),
      threshold,
      operators.slice(0, threshold)
    ),
  };

  const isCurrentOperators = await contract.view("validate_proof", args);

  t.is(isCurrentOperators, true);
});

test("reject the proof if weights are not matching the threshold", async (t) => {
  const { contract } = t.context.accounts;

  const data = "0x123abc123abc";

  const message = ethers.utils.hashMessage(
    ethers.utils.arrayify(ethers.utils.keccak256(data))
  );

  let args = {
    message_hash: message,
    proof: await Utils.getWeightedSignaturesProof(
      data,
      operators,
      operators.map(() => 1),
      threshold,
      operators.slice(0, threshold - 1)
    ),
  };

  const error = await t.throwsAsync(contract.view("validate_proof", args));

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Low signature weight
});

test("reject the proof if signatures are invalid", async (t) => {
  const { contract } = t.context.accounts;

  const data = "0x123abc123abc";

  const message = ethers.utils.hashMessage(
    ethers.utils.arrayify(ethers.utils.keccak256(data))
  );

  let args = {
    message_hash: message,
    proof: await Utils.getWeightedSignaturesProof(
      data,
      operators,
      operators.map(() => 1),
      threshold,
      wallets.slice(0, threshold)
    ),
  };

  const error = await t.throwsAsync(contract.view("validate_proof", args));

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Malformed signers
});

test("validate the proof from the recent operators", async (t) => {
  const { contract } = t.context.accounts;

  const data = "0x123abc123abc";

  const message = ethers.utils.hashMessage(
    ethers.utils.arrayify(ethers.utils.keccak256(data))
  );

  const validPreviousOperators = previousOperators.slice(
    -(OLD_KEY_RETENTION - 1)
  );

  t.is(validPreviousOperators.length, OLD_KEY_RETENTION - 1);

  await Promise.all(
    validPreviousOperators.map(async (operators) => {
      const isCurrentOperators = await contract.view("validate_proof", {
        message_hash: message,
        proof: await Utils.getWeightedSignaturesProof(
          data,
          operators,
          operators.map(() => 1),
          threshold,
          operators.slice(0, threshold)
        ),
      });

      t.is(isCurrentOperators, false);
    })
  );
});

test("reject the proof from the operators older than key retention", async (t) => {
  const { contract } = t.context.accounts;

  const data = "0x123abc123abc";

  const message = ethers.utils.hashMessage(
    ethers.utils.arrayify(ethers.utils.keccak256(data))
  );

  const invalidPreviousOperators = previousOperators.slice(
    0,
    -(OLD_KEY_RETENTION - 1)
  );

  await Promise.all(
    invalidPreviousOperators.map(async (operators) => {
      const error = await t.throwsAsync(
        contract.view("validate_proof", {
          message_hash: message,
          proof: await Utils.getWeightedSignaturesProof(
            data,
            operators,
            operators.map(() => 1),
            threshold,
            operators.slice(0, threshold)
          ),
        })
      );

      // t.log(error?.message); // uncomment to see the error message

      t.not(error, undefined); // Invalid operators
    })
  );
});

test("validate the proof for a single operator", async (t) => {
  const { contract, root } = t.context.accounts;

  const singleOperator = getAddresses([owner]);

  const didTransferOperatorship = await root.call(
    contract,
    "transfer_operatorship",
    {
      params: await Utils.getTransferWeightedOperatorshipCommand(
        singleOperator,
        [1],
        1
      ),
    },
    { attachedDeposit: "0" }
  );

  t.is(didTransferOperatorship, true);

  const data = "0x123abc123abc";

  const message = ethers.utils.hashMessage(
    ethers.utils.arrayify(ethers.utils.keccak256(data))
  );

  let args = {
    message_hash: message,
    proof: await Utils.getWeightedSignaturesProof(data, [owner], [1], 1, [
      owner,
    ]),
  };

  const isCurrentOperators = await contract.view("validate_proof", args);

  t.is(isCurrentOperators, true);
});

test("validate the proof for a single signer", async (t) => {
  const { contract, root } = t.context.accounts;

  const didTransferOperatorship = await root.call(
    contract,
    "transfer_operatorship",
    {
      params: await Utils.getTransferWeightedOperatorshipCommand(
        getAddresses(operators),
        operators.map(() => 1),
        1
      ),
    },
    { attachedDeposit: "0" }
  );

  t.is(didTransferOperatorship, true);

  const data = "0x123abc123abc";

  const message = ethers.utils.hashMessage(
    ethers.utils.arrayify(ethers.utils.keccak256(data))
  );

  let args = {
    message_hash: message,
    proof: await Utils.getWeightedSignaturesProof(
      data,
      operators,
      operators.map(() => 1),
      1,
      operators.slice(0, 1)
    ),
  };

  const isCurrentOperators = await contract.view("validate_proof", args);

  t.is(isCurrentOperators, true);
});

test("should allow owner to transfer operatorship", async (t) => {
  const { contract, root } = t.context.accounts;

  const newOperators = [
    "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
    "0xb7900E8Ec64A1D1315B6D4017d4b1dcd36E6Ea88",
  ];

  const didTransferOperatorship = await root.call(
    contract,
    "transfer_operatorship",
    {
      params: await Utils.getTransferWeightedOperatorshipCommand(
        newOperators,
        [1, 1],
        2
      ),
    },
    { attachedDeposit: "0" }
  );

  t.is(didTransferOperatorship, true);
});

test("should not allow transferring operatorship to address zero", async (t) => {
  const { contract, root } = t.context.accounts;

  const newOperators = [
    ADDRESS_ZERO,
    "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
  ];

  const error = await t.throwsAsync(
    root.call(
      contract,
      "transfer_operatorship",
      {
        params: await Utils.getTransferWeightedOperatorshipCommand(
          newOperators,
          [1, 1],
          2
        ),
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid operators
});

test("should not allow transferring operatorship to duplicated operators", async (t) => {
  const { contract, root } = t.context.accounts;

  const newOperators = [
    "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
    "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
  ];

  const error = await t.throwsAsync(
    root.call(
      contract,
      "transfer_operatorship",
      {
        params: await Utils.getTransferWeightedOperatorshipCommand(
          newOperators,
          [1, 1],
          2
        ),
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid operators
});

test("should not allow transferring operatorship to unsorted operators", async (t) => {
  const { contract, root } = t.context.accounts;

  const newOperators = [
    "0xb7900E8Ec64A1D1315B6D4017d4b1dcd36E6Ea88",
    "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
  ];

  const error = await t.throwsAsync(
    root.call(
      contract,
      "transfer_operatorship",
      {
        params: await ethers.utils.defaultAbiCoder.encode(
          ["address[]", "uint256[]", "uint256"],
          [newOperators, [1, 1], 2]
        ),
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid operators
});

test("should not allow operatorship transfer to the previous operators", async (t) => {
  const { contract, root } = t.context.accounts;

  const updatedOperators = getAddresses(operators.slice(0, threshold));

  const didTransferOperatorship = await root.call(
    contract,
    "transfer_operatorship",
    {
      params: await Utils.getTransferWeightedOperatorshipCommand(
        updatedOperators,
        updatedOperators.map(() => 1),
        threshold
      ),
    },
    { attachedDeposit: "0" }
  );

  t.is(didTransferOperatorship, true);

  const oldOperators = getAddresses(operators);

  const error = await t.throwsAsync(
    root.call(
      contract,
      "transfer_operatorship",
      {
        params: await Utils.getTransferWeightedOperatorshipCommand(
          oldOperators,
          oldOperators.map(() => 1),
          threshold
        ),
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Duplicate operators
});

test("should not allow transferring operatorship with invalid threshold", async (t) => {
  const { contract, root } = t.context.accounts;

  const newOperators = [
    "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
    "0xb7900E8Ec64A1D1315B6D4017d4b1dcd36E6Ea88",
  ];

  let error = await t.throwsAsync(
    root.call(
      contract,
      "transfer_operatorship",
      {
        params: await Utils.getTransferWeightedOperatorshipCommand(
          newOperators,
          [1, 1],
          0
        ),
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid threshold

  error = await t.throwsAsync(
    root.call(
      contract,
      "transfer_operatorship",
      {
        params: await Utils.getTransferWeightedOperatorshipCommand(
          newOperators,
          [1, 1],
          3
        ),
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid threshold
});

test("should not allow transferring operatorship with invalid number of weights", async (t) => {
  const { contract, root } = t.context.accounts;

  const newOperators = [
    "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
    "0xb7900E8Ec64A1D1315B6D4017d4b1dcd36E6Ea88",
  ];

  let error = await t.throwsAsync(
    root.call(
      contract,
      "transfer_operatorship",
      {
        params: await Utils.getTransferWeightedOperatorshipCommand(
          newOperators,
          [1],
          0
        ),
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid weights

  error = await t.throwsAsync(
    root.call(
      contract,
      "transfer_operatorship",
      {
        params: await Utils.getTransferWeightedOperatorshipCommand(
          newOperators,
          [1, 1, 1],
          3
        ),
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid weights
});

test("should expose correct hashes and epoch", async (t) => {
  const { contract } = t.context.accounts;

  const operatorsHistory = [...previousOperators, operators];

  await Promise.all(
    operatorsHistory.map(async (operators, i) => {
      const payload = await Utils.getTransferWeightedOperatorshipCommand(
        getAddresses(operators),
        operators.map(() => 1),
        threshold
      );

      const hash = ethers.utils.keccak256(payload);

      const hashForEpoch = await contract.view("hash_for_epoch", {
        epoch: i + 1,
      });

      t.is(hashForEpoch, hash);

      const epochForHash = await contract.view("epoch_for_hash", { hash });

      t.is(epochForHash, i + 1);
    })
  );
});

// Gateway Tests

test("should fail if chain id mismatches", async (t) => {
  const { contract, root } = t.context.accounts;

  const data = await Utils.buildCommandBatch(
    CHAIN_ID + 1,
    [Utils.getRandomID()],
    ["transferOperatorship"],
    [
      await Utils.getTransferWeightedOperatorshipCommand(
        [
          "0xb7900E8Ec64A1D1315B6D4017d4b1dcd36E6Ea88",
          "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
        ],
        [1, 1],
        2
      ),
    ]
  );

  const message = ethers.utils.hashMessage(
    ethers.utils.arrayify(ethers.utils.keccak256(data))
  );

  const input = await Utils.getSignedWeightedExecuteInput(
    data,
    operators,
    operators.map(() => 1),
    threshold,
    operators.slice(0, threshold)
  );

  const error = await t.throwsAsync(
    root.call(
      contract,
      "execute",
      {
        message_hash: message,
        input,
      },
      { attachedDeposit: "0" }
    )
  );

  t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid chain id
});
