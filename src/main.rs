use color_eyre::Report;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::SystemTime;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct ModemIp {
    modem: String,
    downstreamSpeed: String,
    modemidx: String,
    upstreamSpeed: String,
    Found_ON_DHCPserver: String,
    ClientIP: Ipv4Addr,
    cf: String,
    encodedmac: String,
    modemIP: Ipv4Addr,
    CMTS: String,
    cpeMAC: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct MacMapData {
    NODE: String,
    RcvPwr: (),
    PrimDS: String,
    nodeQuery: String,
    SYSDESC: String,
    Updated: String,
    CHASSIS_MODEL: String,
    OnlineStatus: String,
    Current: String,
    PrimDSIDX: String,
    visitorIP: Ipv4Addr,
    CableIF: String,
    PrimUS: String,
    IP: Ipv4Addr,
    MacDomain: String,
    MAC: String,
    PrimUSIDX: String,
    cmts: String,
    MODEMIDX: u32,
}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct MacMap {
    macmapData: MacMapData,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct UpstreamData {
    yUncorr: u32,
    xGood: u32,
    zCorr: u32,
    SNR: String,
    #[serde(rename = "Channel Frequency")]
    ChannelFrequency: String,
    #[serde(rename = "Upstream Pwr")]
    UpstreamPwr: String,
    #[serde(rename = "Channel Width")]
    ChannelWidth: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct DocsisMode {
    docsisMode: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct UsStats {
    upstreamData: Vec<UpstreamData>,
    DOCS_IF_MIB_docsIf3UsChSetChList: String,
    DOCS_IF_MIB_docsIfCmtsCmPtr: String,
    CM_Estimated_CNIR: Vec<u32>,
    CMTS_Upstream_Idx: Vec<String>,
    CM_OFFset_Power: Vec<u32>,
    tcsIndex: u32,
    portSpec: DocsisMode,
    UpstreamInts: String,
    avgReturnPwr: f32,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct DownstreamData {
    #[serde(rename = "Channel Frequency")]
    ChannelFrequency: String,
    Uncorr: String,
    zCorr: String,
    #[serde(rename = "DownStream SNR")]
    DownStreamSNR: String,
    #[serde(rename = "DownStream Pwr")]
    DownStreamPwr: String,
    #[serde(rename = "Good x10000")]
    Goodx10000: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct DsStats {
    dscount: u32,
    #[serde(rename = "Standard_Deviation")]
    StandardDeviation: String,
    Variance: String,
    QAMcount: u32,
    avgSNR: f32,
    avgPwr: f32,
    DownStreams: Vec<DownstreamData>,
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup()?;

    let client = reqwest::Client::new();
    let modem_data = client
        .get("http://ma.speedtest.rcn.net/lookup_mip_merlin-new.cgi")
        .query(&[("_", cachebuster())])
        .send()
        .await?
        .json::<ModemIp>()
        .await?;

    let macmap = client
        .get("http://ma.speedtest.rcn.net/merlin/macmap_Ver2.cgi")
        .query(&[
            ("_", format!("{}", cachebuster())),
            ("mac", modem_data.modem.replace(":", "")),
        ])
        .send()
        .await?
        .json::<MacMap>()
        .await?;

    let us_response = client
        .get("http://ma.speedtest.rcn.net/merlin/rfmodem_us_Ver2.cgi")
        .query(&[
            ("_", cachebuster().to_string()),
            ("mac", modem_data.modem),
            ("INT", macmap.macmapData.CableIF.to_string()),
        ])
        .send();

    let ds_response = client
        .get("http://ma.speedtest.rcn.net/merlin/rfmodem_ds.cgi")
        .query(&[
            ("_", cachebuster().to_string()),
            ("ip", modem_data.modemIP.to_string()),
        ])
        .send();

    let us = us_response.await?.json::<UsStats>().await?;
    let good: u32 = us.upstreamData.iter().map(|i| i.xGood).sum();
    let uncorrectable: u32 = us.upstreamData.iter().map(|i| i.yUncorr).sum();

    println!("Packet loss: {}", uncorrectable as f32 / good as f32);

    let ds_response = ds_response.await?;

    debug!("downstream response: {ds_response:?}");

    let ds = ds_response.json::<DsStats>().await;

    info!("{ds:?}");

    Ok(())
}

// Generate cache buster constant from system time
fn cachebuster() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
