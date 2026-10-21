use super::Entry;
use super::row::{Row, row};
use crate::station::Station;
use crate::v201::messages::provisioning::report::{DataType, Mutability};
use crate::wire::ci_string::TooLong;

pub const OCPP_COMM_CTRLR: &str = "OCPPCommCtrlr";
pub const HEARTBEAT_INTERVAL: &str = "HeartbeatInterval";
pub const CLOCK_CTRLR: &str = "ClockCtrlr";
pub const DATE_TIME: &str = "DateTime";
pub const DEVICE_DATA_CTRLR: &str = "DeviceDataCtrlr";
pub const ITEMS_PER_MESSAGE: &str = "ItemsPerMessage";
pub const AVAILABILITY_STATE: &str = "AvailabilityState";
pub const TX_CTRLR: &str = "TxCtrlr";
pub const TX_START_POINT: &str = "TxStartPoint";
pub const TX_STOP_POINT: &str = "TxStopPoint";
pub const STOP_TX_ON_EV_SIDE_DISCONNECT: &str = "StopTxOnEVSideDisconnect";
const TX_POINTS: &str = "EVConnected,Authorized,PowerPathClosed,EnergyTransfer";
pub const CONNECTOR_STATUSES: &str = "Available,Occupied,Reserved,Unavailable,Faulted";

pub const ITEMS_PER_GET: usize = 50;
pub const ITEMS_PER_REPORT: usize = 100;
const ITEMS_PER_SET: usize = 50;
const BYTES_PER_MESSAGE: &str = "65000";

pub fn entries(station: &Station) -> Result<Vec<Entry>, TooLong> {
    use DataType::{Boolean, DateTime, Decimal, Integer, MemberList, OptionList, SequenceList};
    use Mutability::{ReadOnly, ReadWrite};
    let phases = station.phases.to_string();
    let (per_get, per_set, per_report) = (
        ITEMS_PER_GET.to_string(),
        ITEMS_PER_SET.to_string(),
        ITEMS_PER_REPORT.to_string(),
    );
    let mut rows = vec![
        row(
            OCPP_COMM_CTRLR,
            "ActiveNetworkProfile",
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
            "NetworkConfigurationPriority",
            "0",
            ReadWrite,
            SequenceList,
        )
        .values("0"),
        row(
            OCPP_COMM_CTRLR,
            "NetworkProfileConnectionAttempts",
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
            "SecurityCtrlr",
            "OrganizationName",
            "",
            ReadWrite,
            DataType::String,
        ),
        row(
            "SecurityCtrlr",
            "CertificateEntries",
            "0",
            ReadOnly,
            Integer,
        ),
        row("SecurityCtrlr", "SecurityProfile", "0", ReadOnly, Integer),
        row("ChargingStation", "Available", "true", ReadOnly, Boolean),
        row(
            "ChargingStation",
            AVAILABILITY_STATE,
            "Available",
            ReadOnly,
            OptionList,
        )
        .values(CONNECTOR_STATUSES),
        row(
            "ChargingStation",
            "SupplyPhases",
            &phases,
            ReadOnly,
            Integer,
        ),
    ];
    let power = format!("{:.0}", station.limit_w());
    for id in 1..=station.connectors.len() as u32 {
        rows.extend([
            row("EVSE", "Available", "true", ReadOnly, Boolean).evse(id, None),
            row(
                "EVSE",
                AVAILABILITY_STATE,
                "Available",
                ReadOnly,
                OptionList,
            )
            .evse(id, None)
            .values(CONNECTOR_STATUSES),
            row("EVSE", "Power", &power, ReadOnly, Decimal)
                .evse(id, None)
                .unit("W"),
            row("EVSE", "SupplyPhases", &phases, ReadOnly, Integer).evse(id, None),
            row("Connector", "Available", "true", ReadOnly, Boolean).evse(id, Some(1)),
            row(
                "Connector",
                AVAILABILITY_STATE,
                "Available",
                ReadOnly,
                OptionList,
            )
            .evse(id, Some(1))
            .values(CONNECTOR_STATUSES),
            row(
                "Connector",
                "ConnectorType",
                "cType2",
                ReadOnly,
                DataType::String,
            )
            .evse(id, Some(1)),
            row("Connector", "SupplyPhases", &phases, ReadOnly, Integer).evse(id, Some(1)),
        ]);
    }
    rows.into_iter().map(Row::entry).collect()
}
