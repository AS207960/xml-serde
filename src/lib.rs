#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate core;

mod de;
mod error;
mod ser;
mod tag;

pub(crate) use tag::Tag;

pub use de::{from_events, from_str, from_string, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_events, to_events_custom, to_string, to_string_custom, Options, Serializer};

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}epp")]
    pub enum EPPMessageType {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}hello", skip_deserializing)]
        Hello {},
        // #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}greeting", skip_serializing)]
        // Greeting(EPPGreeting),
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}command", skip_deserializing)]
        Command(EPPCommand),
        // #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}response", skip_serializing)]
        // Response(EPPResponse),
    }

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    #[serde(transparent)]
    pub struct EPPMessage {
        pub message: EPPMessageType,
    }

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    pub struct EPPCommand {
        #[serde(rename = "$valueRaw")]
        pub command: String,
        #[serde(
            rename = "{urn:ietf:params:xml:ns:epp-1.0}clTRID",
            skip_serializing_if = "Option::is_none"
        )]
        pub client_transaction_id: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    pub enum EPPCommandType {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}login")]
        Login(EPPLogin),
    }

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    pub struct EPPLogin {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}clID")]
        pub client_id: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}pw")]
        pub password: String,
        #[serde(
            rename = "$attr:{http://www.w3.org/2001/XMLSchema-instance}newPW",
            skip_serializing_if = "Option::is_none"
        )]
        pub new_password: Option<String>,
        pub options: EPPLoginOptions,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}svcs")]
        pub services: EPPLoginServices,
    }

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    pub struct EPPLoginOptions {
        pub version: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}lang")]
        pub language: String,
    }

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    pub struct EPPLoginServices {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}objURI")]
        pub objects: Vec<String>,
    }

    #[test]
    fn encode() {
        pretty_env_logger::init();
        assert_eq!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="urn:ietf:params:xml:ns:epp-1.0 epp-1.0.xsd">
  <command>
    <EPPCommand>&<clTRID>&amp;</clTRID>
    </EPPCommand>
  </command>
</epp>"#
            .to_string(),
            super::ser::to_string(&EPPMessage {
                message: EPPMessageType::Command(EPPCommand {
                    command: "&".to_string(),
                    client_transaction_id: Some("&".to_string()),
                }),
            })
            .unwrap(),
        );
    }

    #[test]
    fn decode() {
        pretty_env_logger::init();
        assert_eq!(
            EPPMessage {
                message: EPPMessageType::Command(EPPCommand {
                    command: "&".to_string(),
                    client_transaction_id: Some("&".to_string()),
                }),
            },
            super::de::from_str(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <command>
    <EPPCommand>&<clTRID>&amp;</clTRID>
    </EPPCommand>
  </command>
</epp>
"#,
            )
            .unwrap(),
        );
    }
}
