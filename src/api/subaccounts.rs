//! Subaccount API endpoints.
//!
//! This module provides functions for managing subaccounts, including creation,
//! balance retrieval, and transfers between subaccounts.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        CreateSubaccountRequest, CreateSubaccountResponse, GetSubaccountTransfersParams,
        RestingOrderValueResponse, SubaccountBalancesResponse, SubaccountNettingResponse,
        SubaccountTransfersResponse, TransferBetweenSubaccountsRequest, TransferResponse,
        UpdateSubaccountNettingRequest,
    },
};

/// Creates a numbered subaccount (1-32) for the authenticated user.
pub async fn create_subaccount(
    http: &HttpClient,
    request: CreateSubaccountRequest,
) -> Result<CreateSubaccountResponse> {
    http.post("/portfolio/subaccounts", &request).await
}

/// Transfers funds between subaccounts (use 0 for primary, 1-32 for numbered).
pub async fn transfer_between_subaccounts(
    http: &HttpClient,
    request: TransferBetweenSubaccountsRequest,
) -> Result<TransferResponse> {
    http.post("/portfolio/subaccounts/transfer", &request).await
}

/// Returns balances for all subaccounts including the primary account.
pub async fn get_subaccount_balances(http: &HttpClient) -> Result<SubaccountBalancesResponse> {
    http.get("/portfolio/subaccounts/balances").await
}

/// Returns a paginated list of transfers between subaccounts.
pub async fn get_subaccount_transfers(
    http: &HttpClient,
    params: GetSubaccountTransfersParams,
) -> Result<SubaccountTransfersResponse> {
    let path = format!(
        "/portfolio/subaccounts/transfers{}",
        params.to_query_string()
    );
    http.get(&path).await
}

/// Returns netting configuration for all subaccounts.
pub async fn get_subaccount_netting(http: &HttpClient) -> Result<SubaccountNettingResponse> {
    http.get("/portfolio/subaccounts/netting").await
}

/// Updates the netting configuration for a specific subaccount.
pub async fn update_subaccount_netting(
    http: &HttpClient,
    request: UpdateSubaccountNettingRequest,
) -> Result<()> {
    http.put_no_response("/portfolio/subaccounts/netting", &request)
        .await
}

/// Returns the total value in cents of all resting orders (primarily for FCM members).
pub async fn get_resting_order_value(http: &HttpClient) -> Result<RestingOrderValueResponse> {
    http.get("/portfolio/summary/total_resting_order_value")
        .await
}
