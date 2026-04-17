#![allow(
    clippy::module_name_repetitions,
    reason = "Request suffix is intentional for clarity"
)]

use bon::Builder;
use chrono::NaiveDate;
use serde::Serialize;
use serde_with::{
    DisplayFromStr, StringWithSeparator, formats::CommaSeparator, serde_as, skip_serializing_none,
};

use crate::clob::types::{AssetType, Side, SignatureType, TimeRange};
use crate::types::{Address, B256, U256};

#[serde_as]
#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct MidpointRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
}

#[serde_as]
#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct PriceRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
    pub side: Side,
}

#[non_exhaustive]
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct SpreadRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
    pub side: Option<Side>,
}

#[non_exhaustive]
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct OrderBookSummaryRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
    pub side: Option<Side>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct LastTradePriceRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
}

#[serde_as]
#[non_exhaustive]
#[skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct PriceHistoryRequest {
    /// The CLOB token ID to fetch price history for.
    #[serde_as(as = "DisplayFromStr")]
    pub market: U256,
    /// The time range for the price history query.
    /// Either a predefined interval or explicit start/end timestamps.
    #[serde(flatten)]
    #[builder(into)]
    pub time_range: TimeRange,
    /// Optional fidelity (number of data points).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fidelity: Option<u32>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(on(String, into))]
pub struct CancelMarketOrderRequest {
    /// The market condition ID to cancel orders for.
    pub market: Option<B256>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub asset_id: Option<U256>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Clone, Builder, Serialize)]
#[builder(on(String, into))]
pub struct TradesRequest {
    pub id: Option<String>,
    #[serde(rename = "taker")]
    pub taker_address: Option<Address>,
    #[serde(rename = "maker")]
    pub maker_address: Option<Address>,
    /// The market condition ID to filter trades.
    pub market: Option<B256>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub asset_id: Option<U256>,
    pub before: Option<i64>,
    pub after: Option<i64>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(on(String, into))]
pub struct OrdersRequest {
    #[serde(rename = "id")]
    pub order_id: Option<String>,
    /// The market condition ID to filter orders.
    pub market: Option<B256>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub asset_id: Option<U256>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Serialize, Builder)]
pub struct DeleteNotificationsRequest {
    #[serde(rename = "ids", skip_serializing_if = "Vec::is_empty")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    #[builder(default)]
    pub notification_ids: Vec<String>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Clone, Builder, Serialize)]
#[builder(on(String, into))]
pub struct BalanceAllowanceRequest {
    pub asset_type: AssetType,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub token_id: Option<U256>,
    pub signature_type: Option<SignatureType>,
}

pub type UpdateBalanceAllowanceRequest = BalanceAllowanceRequest;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(on(String, into))]
pub struct UserRewardsEarningRequest {
    pub date: NaiveDate,
    #[builder(default)]
    pub order_by: String,
    #[builder(default)]
    pub position: String,
    #[builder(default)]
    pub no_competition: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToQueryParams as _;
    use crate::types::b256;

    #[test]
    fn trades_request_as_params_should_succeed() {
        let market = b256!("0000000000000000000000000000000000000000000000000000000000010000");
        let request = TradesRequest::builder()
            .market(market)
            .asset_id(U256::from(100))
            .id("aa-bb")
            .maker_address(Address::ZERO)
            .build();

        assert_eq!(
            request.query_params(None),
            "?id=aa-bb&maker=0x0000000000000000000000000000000000000000&market=0x0000000000000000000000000000000000000000000000000000000000010000&asset_id=100"
        );
        assert_eq!(
            request.query_params(Some("1")),
            "?id=aa-bb&maker=0x0000000000000000000000000000000000000000&market=0x0000000000000000000000000000000000000000000000000000000000010000&asset_id=100&next_cursor=1"
        );
    }

    #[test]
    fn orders_request_as_params_should_succeed() {
        let market = b256!("0000000000000000000000000000000000000000000000000000000000010000");
        let request = OrdersRequest::builder()
            .market(market)
            .asset_id(U256::from(100))
            .order_id("aa-bb")
            .build();

        assert_eq!(
            request.query_params(None),
            "?id=aa-bb&market=0x0000000000000000000000000000000000000000000000000000000000010000&asset_id=100"
        );
        assert_eq!(
            request.query_params(Some("1")),
            "?id=aa-bb&market=0x0000000000000000000000000000000000000000000000000000000000010000&asset_id=100&next_cursor=1"
        );
    }

    #[test]
    fn delete_notifications_request_as_params_should_succeed() {
        let empty_request = DeleteNotificationsRequest::builder().build();
        let request = DeleteNotificationsRequest::builder()
            .notification_ids(vec!["1".to_owned(), "2".to_owned()])
            .build();

        assert_eq!(empty_request.query_params(None), "");
        assert_eq!(request.query_params(None), "?ids=1%2C2");
    }

    #[test]
    fn balance_allowance_request_as_params_should_succeed() {
        let request = BalanceAllowanceRequest::builder()
            .asset_type(AssetType::Collateral)
            .token_id(U256::from(1))
            .signature_type(SignatureType::Eoa)
            .build();

        assert_eq!(
            request.query_params(None),
            "?asset_type=COLLATERAL&token_id=1&signature_type=0"
        );
    }

    #[test]
    fn user_rewards_earning_request_as_params_should_succeed() {
        let request = UserRewardsEarningRequest::builder()
            .date(NaiveDate::MIN)
            .build();

        assert_eq!(
            request.query_params(Some("1")),
            "?date=-262143-01-01&order_by=&position=&no_competition=false&next_cursor=1"
        );
    }
}
