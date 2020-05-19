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
        pub extension: Option<EPPResponseExtension>,
//        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}msgQ")]
//        pub message_queue: Option<EPPMessageQueue>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}resData")]
        pub data: Option<EPPResultData>,
        #[serde(rename = "{urn:ietf:params:xml:ns:epp-1.0}trID")]
        pub transaction_id: EPPTransactionIdentifier,
    }

    #[derive(Debug, Deserialize)]
    pub struct EPPResponseExtension {
        #[serde(rename = "$value", default)]
        value: Vec<EPPResponseExtensionType>
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
        EPPContactInfoResult(Box<EPPDomainInfoData>),
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
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}roid", default)]
        pub registry_id: Option<String>,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}status", default)]
        pub statuses: Vec<EPPDomainStatus>,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}registrant")]
        pub registrant: String,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}contact", default)]
        pub contacts: Vec<EPPDomainInfoContact>,
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}ns", default)]
        pub nameservers: Option<EPPDomainInfoNameservers>,
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
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}authInfo")]
        pub auth_info: Option<EPPDomainAuthInfo>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct EPPDomainStatus {
        #[serde(rename = "$attr:s")]
        pub status: EPPDomainStatusType,
        #[serde(rename = "$value")]
        pub message: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum EPPDomainStatusType {
        #[serde(rename = "clientDeleteProhibited")]
        ClientDeleteProhibited,
        #[serde(rename = "clientHold")]
        ClientHold,
        #[serde(rename = "clientRenewProhibited")]
        ClientRenewProhibited,
        #[serde(rename = "clientTransferProhibited")]
        ClientTransferProhibited,
        #[serde(rename = "clientUpdateProhibited")]
        ClientUpdateProhibited,
        #[serde(rename = "inactive")]
        Inactive,
        #[serde(rename = "ok")]
        Ok,
        #[serde(rename = "Granted")]
        Granted,
        #[serde(rename = "pendingCreate")]
        PendingCreate,
        #[serde(rename = "pendingDelete")]
        PendingDelete,
        #[serde(rename = "Terminated")]
        Terminated,
        #[serde(rename = "pendingRenew")]
        PendingRenew,
        #[serde(rename = "pendingTransfer")]
        PendingTransfer,
        #[serde(rename = "pendingUpdate")]
        PendingUpdate,
        #[serde(rename = "serverDeleteProhibited")]
        ServerDeleteProhibited,
        #[serde(rename = "serverHold")]
        ServerHold,
        #[serde(rename = "serverRenewProhibited")]
        ServerRenewProhibited,
        #[serde(rename = "serverTransferProhibited")]
        ServerTransferProhibited,
        #[serde(rename = "serverUpdateProhibited")]
        ServerUpdateProhibited,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct EPPDomainInfoContact {
        #[serde(rename = "$attr:type")]
        pub contact_type: String,
        #[serde(rename = "$value")]
        pub contact_id: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct EPPDomainInfoNameservers {
        #[serde(rename = "$value")]
        pub servers: Vec<EPPDomainInfoNameserver>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum EPPDomainInfoNameserver {
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}domain:hostObj")]
        HostOnly(String),
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}domain:hostAttr")]
        HostAndAddress {
            #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}domain:hostName")]
            host: String,
            #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}domain:hostAddr", default)]
            addresses: Vec<EPPDomainInfoNameserverAddress>,
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

    #[derive(Debug, Deserialize, Serialize)]
    pub struct EPPDomainAuthInfo {
        #[serde(rename = "{urn:ietf:params:xml:ns:domain-1.0}domain:pw", default)]
        pub password: Option<String>,
    }

    #[test]
    fn encode() {
        pretty_env_logger::init();

        let msg = r#"
<?xml version="1.0" encoding="utf-8"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <response>
    <result code="1000">
      <msg>Command completed successfully</msg>
    </result>
    <resData>
      <domain:infData xmlns:domain="urn:ietf:params:xml:ns:domain-1.0">
        <domain:name>haccvoc.de</domain:name>
        <domain:roid>17828397619534_DOMAIN-KEYSYS</domain:roid>
        <domain:status s="ok"/>
        <domain:registrant>P-GGS4362</domain:registrant>
        <domain:contact type="admin">P-RVS7962</domain:contact>
        <domain:contact type="tech">P-GGS4362</domain:contact>
        <domain:contact type="billing">P-GGS4362</domain:contact>
        <domain:ns>
          <domain:hostObj>NS1.AS207960.NET</domain:hostObj>
          <domain:hostObj>NS2.AS207960.NET</domain:hostObj>
        </domain:ns>
        <domain:clID>as207960</domain:clID>
        <domain:crID>EXTERNAL</domain:crID>
        <domain:crDate>2020-05-14T14:55:50.0Z</domain:crDate>
        <domain:upID>as207960</domain:upID>
        <domain:upDate>2020-05-14T17:19:42.0Z</domain:upDate>
        <domain:exDate>2021-05-14T14:55:50.0Z</domain:exDate>
        <domain:trDate>2020-05-14T14:55:50.0Z</domain:trDate>
        <domain:authInfo>
          <domain:pw/>
        </domain:authInfo>
      </domain:infData>
    </resData>
    <extension>
      <secDNS:infData xmlns:secDNS="urn:ietf:params:xml:ns:secDNS-1.1">
        <secDNS:keyData>
          <secDNS:flags>257</secDNS:flags>
          <secDNS:protocol>3</secDNS:protocol>
          <secDNS:alg>13</secDNS:alg>
          <secDNS:pubKey>UIh8VQuVXbUQwCjV4d+ptxKCvtbI6XcAdf9qnL1f21663JotyeXU/sNF6GUz5jutm1nmcrRbKS8DDGRz0fzoHA==</secDNS:pubKey>
        </secDNS:keyData>
      </secDNS:infData>
    </extension>
    <trID>
      <clTRID>7d6a6b86-674d-4906-a87f-54c187a12651</clTRID>
      <svTRID>4fc529b1-9e22-4269-a33c-f7957c80460d</svTRID>
    </trID>
  </response>
</epp> "#;
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