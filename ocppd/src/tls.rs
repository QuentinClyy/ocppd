use crate::protocol::Protocol;
use native_tls::{Certificate, Identity, Protocol as TlsVersion, TlsConnector};
use tokio_tungstenite::Connector;

pub fn connector(protocol: &Protocol) -> Result<Connector, native_tls::Error> {
    let mut builder = TlsConnector::builder();
    builder.min_protocol_version(Some(TlsVersion::Tlsv12));
    for root in protocol.csms_roots() {
        builder.add_root_certificate(Certificate::from_pem(&root)?);
    }
    if let Some(client) = protocol.client_certificate() {
        let identity = Identity::from_pkcs8(&client.chain_pem, &client.key_pkcs8_pem)?;
        builder.identity(identity);
    }
    Ok(Connector::NativeTls(builder.build()?))
}
