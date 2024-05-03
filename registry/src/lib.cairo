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
mod SharpP2PRegistry {
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
    struct WitnessMetadata {
        reward: u256,
        num_of_steps: u256,
        #[key]
        executor: ContractAddress,
        #[key]
        delegator: ContractAddress,
    }

    #[abi(embed_v0)]
    impl SharpP2PRegistryImpl of super::ISharpP2PRegistry<ContractState> {
        fn deposit(ref self: ContractState, amount: u256) {
            let caller = get_caller_address();
            let this_contract = get_contract_address();
            self.token.read().transfer_from(caller, this_contract, amount);
            let prev = self.balances.read(caller);
            self.balances.write(caller, prev + amount);
            self.emit(Deposit { user: caller, amount: amount });
        }

        fn withdraw(ref self: ContractState, amount: u256) {
            let caller = get_caller_address();
            self.token.read().transfer(caller, amount);
            let prev = self.balances.read(caller);
            self.balances.write(caller, prev - amount);
            self.emit(Withdraw { user: caller, amount: amount });
        }

        fn balance(self: @ContractState, account: ContractAddress) -> u256 {
            self.balances.read(account)
        }

        fn verify_job_witness(ref self: ContractState, proof: StarkProofWithSerde) {
            let metadata = _get_metadata(@proof.public_input);

            self.verifier.read().verify_and_register_fact(proof);

            let fee = _calculate_fee(metadata.reward, self.fee_factor.read());
            self
                .token
                .read()
                .transfer_from(metadata.delegator, metadata.executor, metadata.reward - fee);
            self.token.read().transfer_from(metadata.delegator, self.fee_account.read(), fee);

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

    fn _get_metadata(public_input: @PublicInputWithSerde) -> WitnessMetadata {
        WitnessMetadata {
            reward: 0x0,
            num_of_steps: 0x0,
            executor: 0x0.try_into().unwrap(),
            delegator: 0x0.try_into().unwrap(),
        }
    }

    fn _calculate_fee(amount: u256, fee_factor: u256) -> u256 {
        amount
            * fee_factor
            / u256 {
                low: 0xffffffffffffffffffffffffffffffff_u128,
                high: 0xffffffffffffffffffffffffffffffff_u128
            }
    }
}
