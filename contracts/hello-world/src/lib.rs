#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct ImageJob {
    pub id: u64,
    pub requester: Address,
    pub image_hash: String,
    pub operation: String,
    pub params: String,
    pub status: String,
    pub result_hash: String,
    pub timestamp: u64,
}

#[contracttype]
pub enum JobKey {
    Count,
    Job(u64),
}

#[contract]
pub struct RustVisionProcessor;

#[contractimpl]
impl RustVisionProcessor {
    pub fn submit_job(env: Env, requester: Address, image_hash: String, operation: String, params: String) -> u64 {
        let mut count: u64 = env.storage().instance().get(&JobKey::Count).unwrap_or(0);
        count = count.saturating_add(1);
        env.storage().instance().set(&JobKey::Count, &count);

        let j = ImageJob {
            id: count,
            requester: requester.clone(),
            image_hash,
            operation,
            params,
            status: String::from_str(&env, "pending"),
            result_hash: String::from_str(&env, ""),
            timestamp: env.ledger().timestamp(),
        };
        env.storage().instance().set(&JobKey::Job(count), &j);
        count
    }

    pub fn submit_result(env: Env, worker: Address, job_id: u64, result_hash: String, success: bool) {
        let mut j: ImageJob = env.storage().instance().get(&JobKey::Job(job_id)).expect("job not found");
        assert!(j.status != String::from_str(&env, "done"), "already done");
        if success {
            j.status = String::from_str(&env, "done");
            j.result_hash = result_hash;
        } else {
            j.status = String::from_str(&env, "failed");
        }
        env.storage().instance().set(&JobKey::Job(job_id), &j);
        let _ = worker;
    }

    pub fn view_job(env: Env, job_id: u64) -> ImageJob {
        env.storage().instance().get(&JobKey::Job(job_id)).expect("job not found")
    }
}
