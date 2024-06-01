## Registry Contract

The Registry contract is a critical component of the SHARP-P2P network, responsible for the following tasks:

1. **Fund Management**: Delegators deposit funds into the Registry to offer rewards for job execution. The Registry securely holds these funds until they are distributed to Executors.

2. **Verification and Reward Distribution**: The Registry verifies the submitted Job Witnesses from Executors. This involves checking the Delegator's signature, the job's hash, and the proof of correct execution. If the job is verified successfully, the Registry distributes the corresponding reward to the Executor.

3. **Event Emission**: The Registry emits events upon successful job verification and reward distribution. Delegators can subscribe to these events to receive notifications about the status of their jobs.

4. **Security**: The Registry ensures that only valid and correctly executed jobs are rewarded. This provides a layer of trust and security for both Delegators and Executors in the network.

By managing the financial and verification aspects, the Registry contract plays a pivotal role in maintaining the integrity and efficiency of the Sharp p2p network.