## Architecture and Workflow

We will explore the job pathway as it goes from Delegator to Executor via the peer-to-peer network to better understand the workings of this service:

1. **Delegator Initiates Job**: The Delegator, who has funds deposited into the Registry and is part of the Zetina network, initiates a job. They define the job, which consists of a Job Header and a Job Body. The Job Header contains basic information about the task, such as its reward and its hash, while the Job Body contains the Cairo program to be executed.

2. **Storage in Peer-to-Peer Network**: The Delegator signs and stores the Job Body in a Distributed Hash Table (DHT) within the peer-to-peer network. Simultaneously, they sign the Job Header and send it to the peer-to-peer network using a gossip-sub message.

3. **Executor's Perspective**: An Executor within the network receives the Job Header from the Delegator. The Executor first verifies the validity of the message by checking if the Job Header was correctly signed by the Delegator. If valid, the Executor retrieves the Job Body from the DHT and performs similar checks.

4. **Account Solvency Check**: The Executor also checks if the Delegator's account has a safe amount of funds remaining for the job reward. If everything checks out, the Executor proceeds to execute the job.

5. **Job Execution**: The execution process consumes time and resources on the host machine. Once completed, the Executor produces a Job Witness containing the results of the Cairo program and proof of its correct execution.

6. **Submission to Registry and Receiving Reward**: The Executor promptly submits the job to the Registry. The Registry verifies the signature of the Delegator, the hash of the job, and the job proof of correctness. If all criteria are met, the Registry sends the funds as a reward. The Delegator subscribes to the Registry contract events and, when a job is successfully registered, the output of the job is emitted.

To sum up, the Delegator pays the Executor for doing their job in a way that is safe and sound for both parties of the Zetina network.

![Architecture](architecture.svg)