//! Subaccount API endpoints.
//!
//! This module provides functions for managing subaccounts, including creation,
//! balance retrieval, and transfers between subaccounts.

use crate::{
    client::HttpClient,
    error::Result,
    models::{
        CreateSubaccountRequest, CreateSubaccountResponse, GetSubaccountTransfersParams,
        RestingOrderValueResponse, SubaccountBalancesResponse, SubaccountTransfersResponse,
        TransferBetweenSubaccountsRequest, TransferResponse,
    },
};

/// Create a new subaccount.
///
/// Creates a new numbered subaccount for the authenticated user.
/// Subaccounts are numbered sequentially starting from 1, up to a maximum of 32.
pub async fn create_subaccount(
    http: &HttpClient,
    request: CreateSubaccountRequest,
) -> Result<CreateSubaccountResponse> {
    http.post("/portfolio/subaccounts", &request).await
}

/// Transfer funds between subaccounts.
///
/// Transfers funds between the authenticated user's subaccounts.
/// Use 0 for the primary account, or 1-32 for numbered subaccounts.
pub async fn transfer_between_subaccounts(
    http: &HttpClient,
    request: TransferBetweenSubaccountsRequest,
) -> Result<TransferResponse> {
    http.post("/portfolio/subaccounts/transfer", &request).await
}

/// Get balances for all subaccounts.
///
/// Returns the balance for all subaccounts including the primary account.
pub async fn get_subaccount_balances(http: &HttpClient) -> Result<SubaccountBalancesResponse> {
    http.get("/portfolio/subaccounts/balances").await
}

/// Get subaccount transfer history.
///
/// Returns a paginated list of all transfers between subaccounts.
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

/// Get total resting order value.
///
/// Returns the total value in cents of all resting orders.
/// This endpoint is primarily intended for FCM members.
pub async fn get_resting_order_value(http: &HttpClient) -> Result<RestingOrderValueResponse> {
    http.get("/portfolio/summary/total_resting_order_value")
        .await
}
