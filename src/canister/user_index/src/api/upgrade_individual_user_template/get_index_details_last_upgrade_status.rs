use ic_cdk_macros::query;

use crate::{data_model::canister_upgrade::UpgradeStatus, CANISTER_DATA};

#[query]
fn get_index_details_last_upgrade_status() -> UpgradeStatus {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .last_run_upgrade_status
            .clone()
    })
}
