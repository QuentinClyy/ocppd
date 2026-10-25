# OCPP 2.0.1

OCPP 2.0.1 (edition 4), every certification profile except ISO 15118 support:

- **Core:**
  - provisioning: boot, the device model, base and custom reports, reset;
  - authorization;
  - transactions, with configurable start and stop points;
  - meter values;
  - availability;
  - remote control and triggers;
  - firmware update and logs;
  - customer information;
  - data transfer.
- **Advanced Security:**
  - security profiles 1 to 3;
  - CSMS and manufacturer certificates;
  - station certificate signing and renewal;
  - network profiles;
  - security events.
- **Local Authorization List Management:** the local list and the
  authorization cache.
- **Smart Charging:**
  - SetChargingProfile, GetChargingProfiles, ClearChargingProfile and
    GetCompositeSchedule;
  - remote start with a profile;
  - charging limits that follow the profiles.
- **Reservation:** of an EVSE, of any EVSE, or of a connector type, with
  expiry.
- **Advanced Device Management:** variable monitors (thresholds, delta,
  periodic), monitoring level and base, and monitoring reports.
- **Advanced User Interface:** display messages, and running and final costs.

## Not supported

- The ISO 15118 messages.
- External charging limits: NotifyChargingLimit and ClearedChargingLimit.
- Reset of a single EVSE (answered `Rejected`).
- The DataSigned and ParkingBayOccupancy transaction points.
- DisableRemoteAuthorization and DisablePostAuthorize.
- Hard-wired monitors other than the connector lock's, and preconfigured
  monitors.
- Vendor DataTransfer (always `UnknownVendorId`).
- Attribute types other than `Actual`.

## Known gaps

- A ChargingStationMaxProfile is split equally between the charging EVSEs; a
  Relative schedule counts from the transaction start.
- SetNetworkProfile never answers `Failed`, and AllowSecurityProfileDowngrade is
  not offered.
- The clock is the host's, not the CSMS's `currentTime`.
- `BytesPerMessage` limits are reported but not enforced.
