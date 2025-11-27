#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

const GUARD_NS: Symbol = symbol_short!("GUARD");

#[contracttype]
#[derive(Clone)]
pub struct GuardianConfig {
    pub owner: Address,
    pub guardians: Vec<Address>,
    pub threshold: u32,        // how many guardians must approve
    pub pending_new_owner: Option<Address>,
    pub approvals: Vec<Address>,
}

#[contract]
pub struct SecureGuardianContract;

#[contractimpl]
impl SecureGuardianContract {
    // Initialize guardians and threshold for this contract wallet
    pub fn init_guardians(env: Env, owner: Address, guardians: Vec<Address>, threshold: u32) {
        if threshold == 0 {
            panic!("threshold must be > 0");
        }
        if guardians.len() == 0 {
            panic!("need guardians");
        }
        if threshold > guardians.len() as u32 {
            panic!("threshold too high");
        }

        let inst = env.storage().instance();
        if inst.has(&GUARD_NS) {
            panic!("already initialized");
        }

        let cfg = GuardianConfig {
            owner,
            guardians,
            threshold,
            pending_new_owner: None,
            approvals: Vec::new(&env),
        };
        inst.set(&GUARD_NS, &cfg);
    }

    // Current owner starts recovery to propose a new owner address
    pub fn start_recovery(env: Env, caller: Address, new_owner: Address) {
        let inst = env.storage().instance();
        let mut cfg: GuardianConfig =
            inst.get(&GUARD_NS).unwrap_or_else(|| panic!("not initialized"));

        if caller != cfg.owner {
            panic!("only owner can start recovery");
        }

        cfg.pending_new_owner = Some(new_owner);
        cfg.approvals = Vec::new(&env);
        inst.set(&GUARD_NS, &cfg);
    }

    // A guardian approves the pending recovery
    pub fn approve_recovery(env: Env, guardian: Address) {
        let inst = env.storage().instance();
        let mut cfg: GuardianConfig =
            inst.get(&GUARD_NS).unwrap_or_else(|| panic!("not initialized"));

        let pending = match cfg.pending_new_owner {
            Some(ref _p) => {}
            None => panic!("no pending recovery"),
        };

        if !Self::is_guardian_internal(&cfg.guardians, &guardian) {
            panic!("not a guardian");
        }

        // avoid duplicate approvals
        if Self::has_approved_internal(&cfg.approvals, &guardian) {
            panic!("already approved");
        }

        cfg.approvals.push_back(guardian);

        if cfg.approvals.len() as u32 >= cfg.threshold {
            // enough approvals: change owner and clear pending
            cfg.owner = cfg.pending_new_owner.clone().unwrap();
            cfg.pending_new_owner = None;
            cfg.approvals = Vec::new(&env);
        }

        inst.set(&GUARD_NS, &cfg);
    }

    // Read current owner
    pub fn get_owner(env: Env) -> Address {
        let inst = env.storage().instance();
        let cfg: GuardianConfig =
            inst.get(&GUARD_NS).unwrap_or_else(|| panic!("not initialized"));
        cfg.owner
    }

    // Read guardian list and threshold
    pub fn get_config(env: Env) -> GuardianConfig {
        let inst = env.storage().instance();
        inst.get(&GUARD_NS).unwrap_or_else(|| panic!("not initialized"))
    }

    // Internal helpers
    fn is_guardian_internal(list: &Vec<Address>, addr: &Address) -> bool {
        for g in list.iter() {
            if &g == addr {
                return true;
            }
        }
        false
    }

    fn has_approved_internal(list: &Vec<Address>, addr: &Address) -> bool {
        for a in list.iter() {
            if &a == addr {
                return true;
            }
        }
        false
    }
}
