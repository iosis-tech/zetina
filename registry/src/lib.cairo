use cairo_verifier::StarkProofWithSerde;
use starknet::ContractAddress;

#[starknet::interface]
pub trait ISharpP2PRegistry<TContractState> {
    fn deposit(ref self: TContractState, amount: u256);
    fn withdraw(ref self: TContractState, amount: u256);
    fn balance(self: @TContractState, account: ContractAddress) -> u256;
    fn verify_job_witness(ref self: TContractState, proof: StarkProofWithSerde);
}

#[starknet::interface]
pub trait IFactRegistry<TContractState> {
    fn verify_and_register_fact(ref self: TContractState, stark_proof: StarkProofWithSerde);
    fn verify_and_register_fact_from_contract(
        ref self: TContractState, contract_address: ContractAddress
    );
    fn is_valid(self: @TContractState, fact: felt252) -> bool;
}

#[starknet::contract]
pub mod SharpP2PRegistry {
    use registry::ISharpP2PRegistry;
    use openzeppelin::token::erc20::interface::IERC20DispatcherTrait;
    use cairo_verifier::{
        StarkProofWithSerde, air::public_input::PublicInput,
        deserialization::stark::PublicInputWithSerde
    };
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherImpl};
    use starknet::ContractAddress;
    use super::{IFactRegistryDispatcher, IFactRegistryDispatcherImpl};
    use starknet::{get_caller_address, get_contract_address};

    const FEE_DIVISOR: u256 = 0xffffffffffffffffffffffffffffffff;
    const REWARD_PUBLIC_MEMORY_OFFSET: u32 = 7;
    const NUM_OF_STEPS_PUBLIC_MEMORY_OFFSET: u32 = 5;
    const EXECUTOR_PUBLIC_MEMORY_OFFSET: u32 = 3;
    const DELEGATOR_PUBLIC_MEMORY_OFFSET: u32 = 1;

    #[storage]
    struct Storage {
        token: IERC20Dispatcher,
        fee_account: ContractAddress,
        fee_factor: u256,
        verifier: IFactRegistryDispatcher,
        balances: LegacyMap::<ContractAddress, u256>,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        token_address: ContractAddress,
        fee_account: ContractAddress,
        fee_factor: u256,
        verifier_address: ContractAddress
    ) {
        self.token.write(IERC20Dispatcher { contract_address: token_address });
        self.fee_account.write(fee_account);
        self.fee_factor.write(fee_factor);
        self.verifier.write(IFactRegistryDispatcher { contract_address: verifier_address })
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        Deposit: Deposit,
        Withdraw: Withdraw,
        JobWitnessVerified: WitnessMetadata,
    }

    #[derive(Drop, starknet::Event)]
    struct Deposit {
        #[key]
        user: ContractAddress,
        amount: u256,
    }

    #[derive(Drop, starknet::Event)]
    struct Withdraw {
        #[key]
        user: ContractAddress,
        amount: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct WitnessMetadata {
        pub reward: u256,
        pub num_of_steps: u256,
        #[key]
        pub executor: ContractAddress,
        #[key]
        pub delegator: ContractAddress,
    }

    #[abi(embed_v0)]
    impl SharpP2PRegistryImpl of super::ISharpP2PRegistry<ContractState> {
        fn deposit(ref self: ContractState, amount: u256) {
            let caller = get_caller_address();
            let this_contract = get_contract_address();
            self.token.read().transfer_from(caller, this_contract, amount);
            let prev_balance = self.balances.read(caller);
            self.balances.write(caller, prev_balance + amount);
            self.emit(Deposit { user: caller, amount: amount });
        }

        fn withdraw(ref self: ContractState, amount: u256) {
            let caller = get_caller_address();
            self.token.read().transfer(caller, amount);
            let prev_balance = self.balances.read(caller);
            self.balances.write(caller, prev_balance - amount);
            self.emit(Withdraw { user: caller, amount: amount });
        }

        fn balance(self: @ContractState, account: ContractAddress) -> u256 {
            self.balances.read(account)
        }

        fn verify_job_witness(ref self: ContractState, proof: StarkProofWithSerde) {
            let metadata = get_metadata(@proof.public_input);

            self.verifier.read().verify_and_register_fact(proof);

            let fee = metadata.reward * self.fee_factor.read() / FEE_DIVISOR;
            let remaining_reward = metadata.reward - fee;

            let token = self.token.read();
            token.transfer_from(metadata.delegator, metadata.executor, remaining_reward);
            token.transfer_from(metadata.delegator, self.fee_account.read(), fee);

            self
                .emit(
                    WitnessMetadata {
                        reward: metadata.reward,
                        num_of_steps: metadata.num_of_steps,
                        executor: metadata.executor,
                        delegator: metadata.delegator,
                    }
                );
        }
    }

    pub fn get_metadata(public_input: @PublicInputWithSerde) -> WitnessMetadata {
        let main_page = public_input.main_page.span();
        let main_page_len = main_page.len();
        WitnessMetadata {
            reward: (*main_page.at(main_page_len - REWARD_PUBLIC_MEMORY_OFFSET)).into(),
            num_of_steps: (*main_page.at(main_page_len - NUM_OF_STEPS_PUBLIC_MEMORY_OFFSET)).into(),
            executor: (*main_page.at(main_page_len - EXECUTOR_PUBLIC_MEMORY_OFFSET))
                .try_into()
                .unwrap(),
            delegator: (*main_page.at(main_page_len - DELEGATOR_PUBLIC_MEMORY_OFFSET))
                .try_into()
                .unwrap(),
        }
    }
}
