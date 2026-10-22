use super::row::{Row, row};
use super::table::{AUTH_CTRLR, MASTER_PASS_GROUP_ID};
use crate::protocol::v201::charging_station::local_auth::cache::CAPACITY as CACHE_CAPACITY;
use crate::protocol::v201::charging_station::local_auth::list::{
    CAPACITY as LIST_CAPACITY, ITEMS_PER_MESSAGE,
};
use crate::protocol::v201::charging_station::local_auth::{
    AUTH_CACHE_CTRLR, ENABLED, ENTRIES, LOCAL_AUTH_LIST_CTRLR,
};
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};

pub fn rows() -> Vec<Row<'static>> {
    use DataType::{Boolean, Integer};
    use Mutability::{ReadOnly, ReadWrite};
    let per_message = ITEMS_PER_MESSAGE.to_string();
    vec![
        row(
            AUTH_CTRLR,
            MASTER_PASS_GROUP_ID,
            "",
            ReadWrite,
            DataType::String,
        ),
        row(
            AUTH_CTRLR,
            "AuthorizeRemoteStart",
            "false",
            ReadWrite,
            Boolean,
        ),
        row(
            AUTH_CTRLR,
            "LocalAuthorizeOffline",
            "true",
            ReadWrite,
            Boolean,
        ),
        row(AUTH_CTRLR, "LocalPreAuthorize", "true", ReadWrite, Boolean),
        row(
            AUTH_CTRLR,
            "OfflineTxForUnknownIdEnabled",
            "false",
            ReadWrite,
            Boolean,
        ),
        row(AUTH_CACHE_CTRLR, ENABLED, "true", ReadWrite, Boolean),
        row(AUTH_CACHE_CTRLR, "Available", "true", ReadOnly, Boolean),
        row(AUTH_CACHE_CTRLR, "LifeTime", "86400", ReadWrite, Integer)
            .unit("s")
            .min(0.0),
        row(AUTH_CACHE_CTRLR, ENTRIES, "0", ReadOnly, Integer).max(CACHE_CAPACITY as f64),
        row(LOCAL_AUTH_LIST_CTRLR, ENABLED, "true", ReadWrite, Boolean),
        row(
            LOCAL_AUTH_LIST_CTRLR,
            "Available",
            "true",
            ReadOnly,
            Boolean,
        ),
        row(LOCAL_AUTH_LIST_CTRLR, ENTRIES, "0", ReadOnly, Integer).max(LIST_CAPACITY as f64),
        row(
            LOCAL_AUTH_LIST_CTRLR,
            "ItemsPerMessage",
            &per_message,
            ReadOnly,
            Integer,
        ),
        row(
            LOCAL_AUTH_LIST_CTRLR,
            "BytesPerMessage",
            "65000",
            ReadOnly,
            Integer,
        ),
    ]
}
