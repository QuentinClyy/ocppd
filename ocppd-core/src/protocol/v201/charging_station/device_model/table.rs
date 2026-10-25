use super::Entry;
use super::row::{Row, row};
use crate::protocol::v201::charging_station::metering::{
    ALIGNED_DATA_CTRLR, MEASURANDS, SAMPLED_DATA_CTRLR,
};
use crate::protocol::v201::charging_station::provisioning::network_profile::{
    ACTIVE_NETWORK_PROFILE, NETWORK_CONFIGURATION_PRIORITY, NETWORK_PROFILE_CONNECTION_ATTEMPTS,
    SLOTS,
};
use crate::protocol::v201::charging_station::websocket_ping::WEB_SOCKET_PING_INTERVAL;
use crate::protocol::v201::messages::provisioning::report::{DataType, Mutability};
use crate::station::Station;
use crate::wire::ci_string::TooLong;

pub const OCPP_COMM_CTRLR: &str = "OCPPCommCtrlr";
pub const HEARTBEAT_INTERVAL: &str = "HeartbeatInterval";
pub const CLOCK_CTRLR: &str = "ClockCtrlr";
pub const DATE_TIME: &str = "DateTime";
pub const DEVICE_DATA_CTRLR: &str = "DeviceDataCtrlr";
pub const ITEMS_PER_MESSAGE: &str = "ItemsPerMessage";
pub const AVAILABILITY_STATE: &str = "AvailabilityState";
pub const POWER: &str = "Power";
pub const TX_CTRLR: &str = "TxCtrlr";
pub const AUTH_CTRLR: &str = "AuthCtrlr";
pub const MASTER_PASS_GROUP_ID: &str = "MasterPassGroupId";
pub const TX_START_POINT: &str = "TxStartPoint";
pub const TX_STOP_POINT: &str = "TxStopPoint";
pub const STOP_TX_ON_EV_SIDE_DISCONNECT: &str = "StopTxOnEVSideDisconnect";
const ENERGY: &str = "Energy.Active.Import.Register";
const SAMPLED: &str =
    "Energy.Active.Import.Register,Power.Active.Import,Current.Import,Current.Offered,Voltage";
const TX_POINTS: &str = "EVConnected,Authorized,PowerPathClosed,EnergyTransfer";
pub const CONNECTOR_STATUSES: &str = "Available,Occupied,Reserved,Unavailable,Faulted";

pub const ITEMS_PER_GET: usize = 50;
pub const ITEMS_PER_REPORT: usize = 100;
const ITEMS_PER_SET: usize = 50;
pub const BYTES_PER_MESSAGE: &str = "65000";

pub fn entries(station: &Station) -> Result<Vec<Entry>, TooLong> {
    use DataType::{Boolean, DateTime, Integer, MemberList, SequenceList};
    use Mutability::{ReadOnly, ReadWrite};
    let (per_get, per_set, per_report) = (
        ITEMS_PER_GET.to_string(),
        ITEMS_PER_SET.to_string(),
        ITEMS_PER_REPORT.to_string(),
    );
    let mut rows = vec![
        row(
            OCPP_COMM_CTRLR,
            ACTIVE_NETWORK_PROFILE,
            "0",
            ReadOnly,
            Integer,
        ),
        row(OCPP_COMM_CTRLR, "MessageTimeout", "30", ReadOnly, Integer)
            .instance("Default")
            .unit("s"),
        row(
            OCPP_COMM_CTRLR,
            "FileTransferProtocols",
            "HTTP,HTTPS,FTP,FTPS",
            ReadOnly,
            MemberList,
        )
        .values("HTTP,HTTPS,FTP,FTPS"),
        row(
            OCPP_COMM_CTRLR,
            NETWORK_CONFIGURATION_PRIORITY,
            "0",
            ReadWrite,
            SequenceList,
        )
        .values(SLOTS),
        row(
            OCPP_COMM_CTRLR,
            NETWORK_PROFILE_CONNECTION_ATTEMPTS,
            "3",
            ReadWrite,
            Integer,
        )
        .min(0.0),
        row(
            OCPP_COMM_CTRLR,
            "OfflineThreshold",
            "60",
            ReadWrite,
            Integer,
        )
        .unit("s")
        .min(0.0),
        row(OCPP_COMM_CTRLR, "MessageAttempts", "3", ReadWrite, Integer)
            .instance("TransactionEvent")
            .min(0.0),
        row(
            OCPP_COMM_CTRLR,
            "MessageAttemptInterval",
            "60",
            ReadWrite,
            Integer,
        )
        .instance("TransactionEvent")
        .unit("s")
        .min(0.0),
        row(
            OCPP_COMM_CTRLR,
            "UnlockOnEVSideDisconnect",
            "true",
            ReadOnly,
            Boolean,
        ),
        row(OCPP_COMM_CTRLR, "ResetRetries", "0", ReadWrite, Integer).min(0.0),
        row(
            OCPP_COMM_CTRLR,
            WEB_SOCKET_PING_INTERVAL,
            "0",
            ReadWrite,
            Integer,
        )
        .unit("s")
        .min(0.0),
        row(OCPP_COMM_CTRLR, HEARTBEAT_INTERVAL, "0", ReadWrite, Integer)
            .unit("s")
            .min(0.0),
        row(
            DEVICE_DATA_CTRLR,
            ITEMS_PER_MESSAGE,
            &per_report,
            ReadOnly,
            Integer,
        )
        .instance("GetReport"),
        row(
            DEVICE_DATA_CTRLR,
            ITEMS_PER_MESSAGE,
            &per_get,
            ReadOnly,
            Integer,
        )
        .instance("GetVariables"),
        row(
            DEVICE_DATA_CTRLR,
            ITEMS_PER_MESSAGE,
            &per_set,
            ReadOnly,
            Integer,
        )
        .instance("SetVariables"),
        row(
            DEVICE_DATA_CTRLR,
            "BytesPerMessage",
            BYTES_PER_MESSAGE,
            ReadOnly,
            Integer,
        )
        .instance("GetReport"),
        row(
            DEVICE_DATA_CTRLR,
            "BytesPerMessage",
            BYTES_PER_MESSAGE,
            ReadOnly,
            Integer,
        )
        .instance("GetVariables"),
        row(
            DEVICE_DATA_CTRLR,
            "BytesPerMessage",
            BYTES_PER_MESSAGE,
            ReadOnly,
            Integer,
        )
        .instance("SetVariables"),
        row(CLOCK_CTRLR, DATE_TIME, "", ReadOnly, DateTime),
        row(
            CLOCK_CTRLR,
            "TimeSource",
            "Heartbeat",
            ReadWrite,
            SequenceList,
        )
        .values("Heartbeat,RealTimeClock"),
        row(TX_CTRLR, "EVConnectionTimeOut", "60", ReadWrite, Integer)
            .unit("s")
            .min(0.0),
        row(
            TX_CTRLR,
            STOP_TX_ON_EV_SIDE_DISCONNECT,
            "true",
            ReadWrite,
            Boolean,
        ),
        row(
            TX_CTRLR,
            TX_START_POINT,
            "EVConnected,Authorized",
            ReadWrite,
            MemberList,
        )
        .values(TX_POINTS),
        row(
            TX_CTRLR,
            TX_STOP_POINT,
            "EVConnected,Authorized",
            ReadWrite,
            MemberList,
        )
        .values(TX_POINTS),
        row(TX_CTRLR, "StopTxOnInvalidId", "true", ReadWrite, Boolean),
        row(
            SAMPLED_DATA_CTRLR,
            "TxStartedMeasurands",
            ENERGY,
            ReadWrite,
            MemberList,
        )
        .values(MEASURANDS),
        row(
            SAMPLED_DATA_CTRLR,
            "TxUpdatedMeasurands",
            SAMPLED,
            ReadWrite,
            MemberList,
        )
        .values(MEASURANDS),
        row(
            SAMPLED_DATA_CTRLR,
            "TxUpdatedInterval",
            "60",
            ReadWrite,
            Integer,
        )
        .unit("s")
        .min(0.0),
        row(
            SAMPLED_DATA_CTRLR,
            "TxEndedMeasurands",
            ENERGY,
            ReadWrite,
            MemberList,
        )
        .values(MEASURANDS),
        row(
            SAMPLED_DATA_CTRLR,
            "TxEndedInterval",
            "0",
            ReadWrite,
            Integer,
        )
        .unit("s")
        .min(0.0),
        row(
            ALIGNED_DATA_CTRLR,
            "Measurands",
            ENERGY,
            ReadWrite,
            MemberList,
        )
        .values(MEASURANDS),
        row(ALIGNED_DATA_CTRLR, "Interval", "0", ReadWrite, Integer)
            .unit("s")
            .min(0.0),
        row(
            ALIGNED_DATA_CTRLR,
            "TxEndedMeasurands",
            ENERGY,
            ReadWrite,
            MemberList,
        )
        .values(MEASURANDS),
        row(
            ALIGNED_DATA_CTRLR,
            "TxEndedInterval",
            "0",
            ReadWrite,
            Integer,
        )
        .unit("s")
        .min(0.0),
    ];
    rows.extend(super::authorization::rows());
    rows.extend(super::back_off::rows());
    rows.extend(super::security::rows());
    rows.extend(super::display::rows());
    rows.extend(super::monitoring::rows());
    rows.extend(super::reservation::rows());
    rows.extend(super::smart_charging::rows());
    rows.extend(super::tariff_cost::rows());
    rows.extend(super::physical::rows(station));
    rows.into_iter().map(Row::entry).collect()
}
