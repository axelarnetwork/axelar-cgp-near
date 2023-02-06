import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import anyTest, { TestFn } from "ava";
import { sortBy } from "lodash";
import { NEAR, NearAccount, Worker } from "near-workspaces";
import Utils from "./utils";
const { ethers } = require("hardhat");

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

const initContract = async (root: NearAccount, contract: NearAccount) => {
  await contract.deploy(
    "./contract/target/wasm32-unknown-unknown/release/axelar_auth_gateway.wasm"
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

  await initContract(root, contract);

  t.context.worker = worker;
  t.context.accounts = { root, contract, john };
});

test.afterEach.always(async (t) => {
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed to stop the Sandbox:", error);
  });
});

// Auth Tests

test("Auth - validate the proof from the current operators", async (t) => {
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

test("Auth - reject the proof if weights are not matching the threshold", async (t) => {
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

test("Auth - reject the proof if signatures are invalid", async (t) => {
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

test("Auth - validate the proof from the recent operators", async (t) => {
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

test("Auth - reject the proof from the operators older than key retention", async (t) => {
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

test("Auth - validate the proof for a single operator", async (t) => {
  const { contract, root } = t.context.accounts;

  const singleOperator = Utils.getAddresses([owner]);

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

test("Auth - validate the proof for a single signer", async (t) => {
  const { contract, root } = t.context.accounts;

  const didTransferOperatorship = await root.call(
    contract,
    "transfer_operatorship",
    {
      params: await Utils.getTransferWeightedOperatorshipCommand(
        Utils.getAddresses(operators),
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

test("Auth - should allow owner to transfer operatorship", async (t) => {
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

test("Auth - should not allow transferring operatorship to address zero", async (t) => {
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

test("Auth - should not allow transferring operatorship to duplicated operators", async (t) => {
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

test("Auth - should not allow transferring operatorship to unsorted operators", async (t) => {
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

test("Auth - should not allow operatorship transfer to the previous operators", async (t) => {
  const { contract, root } = t.context.accounts;

  const updatedOperators = Utils.getAddresses(operators.slice(0, threshold));

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

  const oldOperators = Utils.getAddresses(operators);

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

test("Auth - should not allow transferring operatorship with invalid threshold", async (t) => {
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

test("Auth - should not allow transferring operatorship with invalid number of weights", async (t) => {
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

test("Auth - should expose correct hashes and epoch", async (t) => {
  const { contract } = t.context.accounts;

  const operatorsHistory = [...previousOperators, operators];

  await Promise.all(
    operatorsHistory.map(async (operators, i) => {
      const payload = await Utils.getTransferWeightedOperatorshipCommand(
        Utils.getAddresses(operators),
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

test("Gateway - should fail if chain id mismatches", async (t) => {
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
        input,
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid chain id
});

test("Gateway - should not allow transferring operatorship to address zero", async (t) => {
  const { contract, root } = t.context.accounts;

  const newOperators = [
    ADDRESS_ZERO,
    "0x6D4017D4b1DCd36e6EA88b7900e8eC64A1D1315b",
  ];

  const data = await Utils.buildCommandBatch(
    CHAIN_ID,
    [Utils.getRandomID()],
    ["transferOperatorship"],
    [
      await Utils.getTransferWeightedOperatorshipCommand(
        newOperators,
        [1, 1],
        2
      ),
    ]
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
        input,
      },
      { attachedDeposit: "0" }
    )
  );

  // t.log(error?.message); // uncomment to see the error message

  t.not(error, undefined); // Invalid operators
});

test("Gateway - should approve and validate contract call", async (t) => {
  const { contract, root } = t.context.accounts;

  const payload = ethers.utils.defaultAbiCoder.encode(
    ["address"],
    [owner.address]
  );
  const payloadHash = ethers.utils.keccak256(payload);
  const commandId = Utils.getRandomID();
  const sourceChain = "Polygon";
  const sourceAddress = "address0x123";
  const sourceTxHash = ethers.utils.keccak256("0x123abc123abc");
  const sourceEventIndex = 17;

  const approveData = await Utils.buildCommandBatch(
    CHAIN_ID,
    [commandId],
    ["approveContractCall"],
    [
      await Utils.getApproveContractCall(
        sourceChain,
        sourceAddress,
        contract.accountId,
        payloadHash,
        sourceTxHash,
        sourceEventIndex
      ),
    ]
  );

  const approveInput = await Utils.getSignedWeightedExecuteInput(
    approveData,
    operators,
    operators.map(() => 1),
    threshold,
    operators.slice(0, threshold)
  );

  const result = await root.call(
    contract,
    "execute",
    {
      input: approveInput,
    },
    { attachedDeposit: "0" }
  );

  t.deepEqual(result, [true]);

  const isApprovedBefore = await contract.view("is_contract_call_approved", {
    command_id: commandId,
    source_chain: sourceChain,
    source_address: sourceAddress,
    contract_address: contract.accountId,
    payload_hash: payloadHash,
  });

  t.is(isApprovedBefore, true);

  await contract.call(
    contract,
    "validate_contract_call",
    {
      command_id: commandId,
      source_chain: sourceChain,
      source_address: sourceAddress,
      payload_hash: payloadHash,
    },
    { attachedDeposit: "0" }
  );

  const isApprovedAfter = await contract.view("is_contract_call_approved", {
    command_id: commandId,
    source_chain: sourceChain,
    source_address: sourceAddress,
    contract_address: contract.accountId,
    payload_hash: payloadHash,
  });

  t.is(isApprovedAfter, false);
});

test("Gateway - call contract", async (t) => {
  const { contract, root } = t.context.accounts;

  const chain = "Polygon";
  const destination = "0xb7900E8Ec64A1D1315B6D4017d4b1dcd36E6Ea88";
  const payload = ethers.utils.defaultAbiCoder.encode(
    ["address", "address"],
    [wallets[1].address, wallets[2].address]
  );

  const event: {
    address: string;
    destination_chain: string;
    destination_contract_address: string;
    payload_hash: string;
    payload: string;
  } = await contract.call(
    contract,
    "call_contract",
    {
      destination_chain: chain,
      destination_contract_address: destination,
      payload,
    },
    { attachedDeposit: "0" }
  );

  t.is(event.address, contract.accountId);
  t.is(event.destination_chain, chain);
  t.is(event.destination_contract_address, destination);
  t.is(event.payload_hash, ethers.utils.keccak256(payload));
  t.is(event.payload, payload);
});
