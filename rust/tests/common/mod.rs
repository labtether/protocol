//! Every payload type in the crate, keyed by its Go name, so a test can pick
//! one from a name found in a fixture or in the Go source.

// Each test file uses part of this module.
#![allow(dead_code)]

use std::collections::BTreeMap;

use labtether_protocol as protocol;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct Payload {
    /// Decodes the JSON into the type and encodes it again.
    pub round_trip: fn(Value) -> Result<Value, String>,
    /// The JSON of the type's default value.
    pub zero: fn() -> Value,
}

fn payload<T: Serialize + DeserializeOwned + Default>() -> Payload {
    Payload {
        round_trip: |value| {
            let decoded: T = serde_json::from_value(value).map_err(|error| error.to_string())?;
            serde_json::to_value(decoded).map_err(|error| error.to_string())
        },
        zero: || serde_json::to_value(T::default()).expect("default value encodes"),
    }
}

macro_rules! payloads {
    ($($name:ident)+) => {
        BTreeMap::from([$((stringify!($name), payload::<protocol::$name>())),+])
    };
}

pub fn payloads() -> BTreeMap<&'static str, Payload> {
    payloads! {
        HeartbeatData ConnectorInfo TelemetryData CommandRequestData CommandResultData
        LogStreamData LogBatchData JournalQueryData JournalEntriesData ConfigUpdateData
        ConfigAppliedData AgentSettingsApplyData AgentSettingsAppliedData
        AgentSettingsStateData UpdateRequestData UpdateProgressData UpdateResultData
        DockerEngineInfo DockerPortMapping DockerMountInfo DockerContainerInfo
        DockerImageInfo DockerNetworkInfo DockerVolumeInfo DockerComposeStack
        DockerDiscoveryData DockerDiscoveryDeltaData DockerContainerStats DockerStatsData
        DockerEventActor DockerEventData DockerActionData DockerActionResultData
        DockerLogsStartData DockerLogsStopData DockerLogsStreamData DockerExecStartData
        DockerExecStartedData DockerExecDataPayload DockerExecInputData DockerExecResizeData
        DockerExecCloseData DockerComposeActionData DockerComposeResultData AlertNotifyData
        EnrollmentApprovedData EnrollmentChallengeData EnrollmentProofData
        EnrollmentRejectedData DockerEndpointTestData DockerEndpointTestResultData
        FileListData FileEntry FileListedData FileReadData FileDataPayload FileWriteData
        FileWrittenData FileMkdirData FileDeleteData FileRenameData FileCopyData
        FileResultData FileSearchData FileSearchResultData PowerActionData PowerResultData
        WoLSendData WoLResultData TerminalProbeResponse TerminalStartData TerminalStartedData
        TerminalDataPayload TerminalResizeData TerminalTmuxKillData TerminalCloseData
        SSHKeyInstallData SSHKeyInstalledData DesktopStartData DesktopStartedData
        DesktopDataPayload DesktopCloseData WebRTCCapabilitiesData WebRTCSessionData
        WebRTCSDPData WebRTCICEData WebRTCStartedData WebRTCStoppedData WebRTCInputData
        ClipboardGetData ClipboardDataPayload ClipboardSetData ClipboardSetAckData
        DesktopAudioStartData DesktopAudioStopData DesktopAudioDataPayload
        DesktopAudioStateData DisplayInfo DisplayListData DesktopDiagnosticRequest
        DesktopDiagnosticData ProcessListData ProcessListedData ProcessInfo ProcessKillData
        ProcessKillResultData ServiceListData ServiceListedData ServiceInfo ServiceActionData
        ServiceResultData DiskListData DiskListedData MountInfo NetworkListData
        NetworkListedData NetInterface NetworkActionData NetworkResultData PackageListData
        PackageListedData PackageInfo PackageActionData PackageResultData CronListData
        CronListedData CronEntry UsersListData UsersListedData UserSession
        WebServiceReportData WebServiceDiscoveryStats WebServiceDiscoverySourceStat
        WebServiceHealthPoint WebServiceHealthSummary DiscoveredWebService
    }
}

/// The closed string types, keyed by Go name, with the values each one lists.
pub fn closed_strings() -> BTreeMap<&'static str, &'static [&'static str]> {
    BTreeMap::from([
        ("PowerAction", protocol::PowerAction::VALUES),
        ("PowerResultStatus", protocol::PowerResultStatus::VALUES),
        ("PowerResultCode", protocol::PowerResultCode::VALUES),
        (
            "DockerEndpointTestStatus",
            protocol::DockerEndpointTestStatus::VALUES,
        ),
        (
            "DockerEndpointTestCode",
            protocol::DockerEndpointTestCode::VALUES,
        ),
        ("PackageInventory", protocol::PackageInventory::VALUES),
    ])
}
