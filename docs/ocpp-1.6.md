# OCPP 1.6

OCPP 1.6J with the Security Whitepaper (edition 4), all profiles:

- **Core:** boot, heartbeat, status notifications, transactions, meter values,
  configuration, remote start and stop, reset, unlock, availability, data
  transfer.
- **Firmware Management:** UpdateFirmware and GetDiagnostics.
- **Local Auth List Management:** SendLocalList, GetLocalListVersion, the
  authorization cache, and offline authorization.
- **Reservation:** ReserveNow and CancelReservation.
- **Smart Charging:** SetChargingProfile, ClearChargingProfile,
  GetCompositeSchedule, and charging limits that follow the profiles.
- **Remote Trigger:** TriggerMessage.
- **Security:**
  - security profiles 0 to 3;
  - certificate management and charge point certificate signing;
  - security events and logs;
  - signed firmware;
  - ExtendedTriggerMessage.

Transaction messages are queued while offline and retried as
`TransactionMessageAttempts` and `TransactionMessageRetryInterval` say.

## Not supported

- Reservations on connector 0 (`ReserveConnectorZeroSupported` is false).
- A main meter.
- Offline transactions for unknown id tags (`AllowOfflineTxForUnknownId` is
  not offered).
- `minChargingRate` is ignored, and the authorization cache has no size limit.

## Known gaps

- An unanswered SignCertificate is not resent. CertificateSigned does not check
  the certificate's subject.
- SetChargingProfile does not validate the schedule's content.
- There is no fallback to the previous security profile when a new one cannot
  connect, and DeleteCertificate never answers `Failed`.
- An offline transaction whose StartTransaction is given up is never reported.
