use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

use candid::{CandidType, Principal};
use ic_cdk::api::{management_canister::provisional::CanisterSettings, time};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Tokens, DEFAULT_SUBACCOUNT};
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::{
        individual_user_template,
        platform_orchestrator::{
            self,
            types::args::{PlatformOrchestratorInitArgs, UpgradeCanisterArg},
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::{
        types::{
            known_principal::{self, KnownPrincipalMap, KnownPrincipalType},
            wasm::WasmType,
        },
        utils::system_time,
    },
    constant::{NNS_CYCLE_MINTING_CANISTER, NNS_LEDGER_CANISTER_ID, YRAL_POST_CACHE_CANISTER_ID},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
        v1::CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    },
};

use crate::known_principal::update_global_known_principal_test::UpgradeStatus;

pub type CanisterId = Principal;

#[test]
fn provision_subnet_orchestrator_canister() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let charlie_global_admin = get_mock_user_charlie_principal_id();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin).unwrap(),
        )
        .unwrap();

    let subnet_orchestrator_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            charlie_global_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    //Check version Installed
    let last_upgrade_status: UpgradeStatus = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_index_details_last_upgrade_status",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let upgrade_status: UpgradeStatus = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            upgrade_status
        })
        .unwrap();

    assert_eq!(last_upgrade_status.version, "v1.0.0");

    //check available capacity on the subnet
    let subnet_available_canister_cnt = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_subnet_available_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let available_capacity: u64 = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            available_capacity
        })
        .unwrap();

    assert!(subnet_available_canister_cnt > 0);

    let individual_user_template_wasm = include_bytes!(
        "../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
    );

    //check if upgrades for individual_canisters_work_fine
    let result = pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "upgrade_canisters_in_network",
            candid::encode_one(UpgradeCanisterArg {
                canister: WasmType::IndividualUserWasm,
                version: "v1.0.1".to_string(),
                wasm_blob: individual_user_template_wasm.to_vec(),
            })
            .unwrap(),
        )
        .map(|res| {
            let res: Result<String, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            res
        })
        .unwrap()
        .unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    //Check version Installed
    let last_upgrade_status: UpgradeStatus = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_index_details_last_upgrade_status",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let upgrade_status: UpgradeStatus = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            upgrade_status
        })
        .unwrap();

    assert!(last_upgrade_status.version.eq("v1.0.1"));
    assert!(last_upgrade_status.successful_upgrade_count > 0);
}
