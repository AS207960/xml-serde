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
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}extension", default)]
        pub extension: Option<Vec<EPPResponseExtensionType>>,
//        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}msgQ")]
//        pub message_queue: Option<EPPMessageQueue>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}resData")]
        pub data: Option<EPPResultData>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}trID")]
        pub transaction_id: EPPTransactionIdentifier,
    }

    #[derive(Debug, Deserialize)]
    pub enum EPPResponseExtensionType {
        #[serde(rename = "{http://www.nominet.org.uk/epp/xml/contact-nom-ext-1.0}infData")]
        NominetContactExtInfo(),
    }

    #[derive(Debug, Deserialize)]
    pub struct EPPResultData {
        #[serde(rename = "$value")]
        pub value: EPPResultDataValue,
    }

    #[derive(Debug, Deserialize)]
    pub enum EPPResultDataValue {
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}infData")]
        EPPDomainInfoResult(Box<EPPDomainInfoData>),
    }

    #[derive(Debug, Deserialize)]
    pub struct EPPTransactionIdentifier {
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}clTRID")]
        pub client_transaction_id: Option<String>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}svTRID")]
        pub server_transaction_id: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct EPPDomainInfoData {
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}name")]
        pub name: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}roid")]
        pub registry_id: String,
//        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}status", default)]
//        pub statuses: Vec<EPPDomainStatus>,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}registrant")]
        pub registrant: String,
//        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}contact", default)]
//        pub contacts: Vec<EPPDomainInfoContact>,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}ns")]
        pub nameservers: EPPDomainInfoNameservers,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}host", default)]
        pub hosts: Vec<String>,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}clID")]
        pub client_id: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}crID")]
        pub client_created_id: Option<String>,
//        #[serde(
//        rename = "{urn:ietf:params:xml:ns:domain-1.0}crDate",
//        deserialize_with = "super::deserialize_datetime_opt",
//        default
//        )]
//        pub creation_date: Option<DateTime<Utc>>,
//        #[serde(
//        rename = "{urn:ietf:params:xml:ns:domain-1.0}exDate",
//        deserialize_with = "super::deserialize_datetime_opt",
//        default
//        )]
//        pub expiry_date: Option<DateTime<Utc>>,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}upID")]
        pub last_updated_client: Option<String>,
//        #[serde(
//        rename = "{urn:ietf:params:xml:ns:domain-1.0}upDate",
//        deserialize_with = "super::deserialize_datetime_opt",
//        default
//        )]
//        pub last_updated_date: Option<DateTime<Utc>>,
//        #[serde(
//        rename = "{urn:ietf:params:xml:ns:domain-1.0}trDate",
//        deserialize_with = "super::deserialize_datetime_opt",
//        default
//        )]
//        pub last_transfer_date: Option<DateTime<Utc>>,
//        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}authInfo")]
//        pub auth_info: Option<EPPDomainAuthInfo>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct EPPDomainInfoNameservers {
        #[serde(rename = "$value")]
        pub servers: Vec<EPPDomainInfoNameserver>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum EPPDomainInfoNameserver {
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}hostObj")]
        HostOnly(String),
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}hostAttr")]
        HostAndAddress {
            #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}hostName")]
            host: String,
            #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}hostAddr")]
            address: EPPDomainInfoNameserverAddress,
        },
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct EPPDomainInfoNameserverAddress {
        #[serde(rename = "$value")]
        pub address: String,
        #[serde(rename = "$attr:ip", default)]
        pub ip_version: EPPDomainInfoNameserverAddressVersion,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum EPPDomainInfoNameserverAddressVersion {
        #[serde(rename = "v4")]
        IPv4,
        #[serde(rename = "v6")]
        IPv6,
    }

    impl std::default::Default for EPPDomainInfoNameserverAddressVersion {
        fn default() -> Self {
            Self::IPv4
        }
    }

    #[test]
    fn encode() {
        pretty_env_logger::init();

        let msg = r#"
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"
      xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:schemaLocation="http://www.nominet.org.uk/epp/xml/epp-1.0 epp-1.0.xsd">
  <response>
    <result code="1000">
      <msg>Command completed successfully</msg>
    </result>
    <resData>
      <domain:infData
        xmlns:domain="urn:ietf:params:xml:ns:domain-1.0"
        xsi:schemaLocation="urn:ietf:params:xml:ns:domain-1.0 domain-1.0.xsd">
        <domain:name>adriana-as207960.co.uk</domain:name>
        <domain:roid>75798252-UK</domain:roid>
        <domain:registrant>XUH5A8W33VVNZH2Q</domain:registrant>
        <domain:ns>
          <domain:hostObj>ns1.adriana-as207960.co.uk.</domain:hostObj>
        </domain:ns>
        <domain:host>ns1.adriana-as207960.co.uk.</domain:host>
        <domain:clID>AS207960</domain:clID>
        <domain:crID>AS207960</domain:crID>
        <domain:crDate>2019-04-23T20:00:08</domain:crDate>
        <domain:exDate>2021-04-22T20:00:08</domain:exDate>
      </domain:infData>
    </resData>
    <extension/>
    <trID>
      <clTRID>440dcdc6-e910-4139-af9e-104c64c97bfb</clTRID>
      <svTRID>15346692879</svTRID>
    </trID>
  </response>
</epp>"#;
        println!("{:?}", super::de::from_str::<EPPMessage>(&msg).unwrap());
        let msg = EPPMessage {
            message: EPPMessageType::Command(EPPCommand {
                command: EPPCommandType::Login(EPPLogin {
                    client_id: "a".to_string(),
                    password: "b".to_string(),
                    new_password: Some("e".to_string()),
                    options: EPPLoginOptions {
                        version: "1.0".to_string(),
                        language: "en".to_string(),
                    },
                    services: EPPLoginServices {
                        objects: vec!["d".to_string()]
                    }
                }),
                client_transaction_id: Some("c".to_string())
            })
        };
        println!("{}", super::ser::to_string(&msg).unwrap());
    }
}