#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod de;
mod ser;
mod error;

pub use ser::{to_string, Serializer};
pub use de::{from_str, Deserializer};
pub use error::{Error, Result};

lazy_static! {
    static ref NAME_RE: regex::Regex = {
        regex::Regex::new(r"^(?:\{(?P<n>.+)\})?(?:(?P<p>.+):)?(?P<e>.+)$").unwrap()
    };
}

#[cfg(test)]
mod tests {

    #[derive(Debug, Serialize, Deserialize)]
    pub enum EPPMessageType {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}hello", skip_deserializing)]
        Hello {},
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}greeting", skip_serializing)]
        Greeting(EPPGreeting),
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}command", skip_deserializing)]
        Command(EPPCommand),
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}response", skip_serializing)]
        Response(EPPResponse),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EPPMessage {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}epp")]
        pub message: EPPMessageType,
    }


    #[derive(Debug, Deserialize)]
    pub struct EPPGreeting {
        #[serde(rename = "$attr:a")]
        pub a: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}svID")]
        pub server_id: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}svDate")]
        pub server_date: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}svcMenu")]
        pub service_menu: EPPServiceMenu,
    }

    #[derive(Debug, Deserialize)]
    pub struct EPPServiceMenu {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}version")]
        pub versions: Vec<String>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}lang")]
        pub languages: Vec<String>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}objURI")]
        pub objects: Vec<String>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}svcExtension")]
        pub extension: Option<EPPServiceExtension>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EPPServiceExtension {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}extURI")]
        pub extensions: Vec<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct EPPCommand {
        #[serde(rename = "$value")]
        pub command: EPPCommandType,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}clTRID", skip_serializing_if = "Option::is_none")]
        pub client_transaction_id: Option<String>,
    }

    #[derive(Debug, Serialize)]
    pub enum EPPCommandType {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}login")]
        Login(EPPLogin),
    }

    #[derive(Debug, Serialize)]
    pub struct EPPLogin {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}clID")]
        pub client_id: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}pw")]
        pub password: String,
        #[serde(rename = "$attr:{http://www.w3.org/2001/XMLSchema-instance}newPW", skip_serializing_if = "Option::is_none")]
        pub new_password: Option<String>,
        pub options: EPPLoginOptions,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}svcs")]
        pub services: EPPLoginServices,
    }

    #[derive(Debug, Serialize)]
    pub struct EPPLoginOptions {
        pub version: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}lang")]
        pub language: String,
    }

    #[derive(Debug, Serialize)]
    pub struct EPPLoginServices {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}objURI")]
        pub objects: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct EPPResponse {
//        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}result")]
//        pub results: Vec<EPPResult>,
//        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}extension", default)]
//        pub extension: Option<EPPResponseExtension>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}msgQ")]
        pub message_queue: Option<EPPMessageQueue>,
//        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}resData")]
//        pub data: Option<EPPResultData>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}trID")]
        pub transaction_id: EPPTransactionIdentifier,
    }

    #[derive(Debug, Deserialize)]
    pub struct EPPMessageQueue {
        #[serde(rename = "$attr:count")]
        pub count: u64,
        #[serde(rename = "$attr:id")]
        pub id: String,
//        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}qDate", deserialize_with = "deserialize_datetime_opt", default)]
//        pub enqueue_date: Option<DateTime<Utc>>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}msg")]
        pub message: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct EPPTransactionIdentifier {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}clTRID")]
        pub client_transaction_id: Option<String>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}svTRID")]
        pub server_transaction_id: Option<String>,
    }

    #[test]
    fn decode() {
        pretty_env_logger::init();

        let msg = r#"
<?xml version="1.0" encoding="utf-8" standalone="no"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
    <response>
        <result code="1301">
            <msg>Command completed successfully; ack to dequeue.</msg>
        </result>
        <msgQ count="1" id="12345"/>
        <resData>
            <domain:trnData xmlns:domain="urn:ietf:params:xml:ns:domain-1.0">
                <domain:name>example.uk.com</domain:name>
                <domain:trStatus>clientApproved</domain:trStatus>
                <domain:reID>H12345</domain:reID>
                <domain:reDate>2011-01-27T23:50:00.0Z</domain:reDate>
                <domain:acID>H54321</domain:acID>
                <domain:acDate>2011-02-01T23:50:00.0Z</domain:acDate>
            </domain:trnData>
        </resData>
        <trID>
            <clTRID>abc123</clTRID>
            <svTRID>321cba</svTRID>
        </trID>
    </response>
</epp>"#;
        println!("{:?}", super::de::from_str::<EPPMessage>(&msg).unwrap());
    }
}