/// Total account ID prefix
pub const PORTFOLIO_TOTAL_ACCOUNT_ID: &str = "TOTAL";

/// Helper function to get the profile-scoped TOTAL account id
pub fn get_total_account_id(profile_id: &str) -> String {
    format!("{}_{}", PORTFOLIO_TOTAL_ACCOUNT_ID, profile_id)
}

/// Cash asset ID prefix
pub const CASH_ASSET_PREFIX: &str = "$CASH";

/// Decimal precision for valuation calculations
pub const DECIMAL_PRECISION: u32 = 6;

/// Decimal precision for display
pub const DISPLAY_DECIMAL_PRECISION: u32 = 2;

/// Quantity threshold for significant positions
pub const QUANTITY_THRESHOLD: &str = "0.00000001";
