//! The closed status and code rules that a receiver relies on to refuse a
//! malformed result. These mirror the Go package's cases.

use labtether_protocol::{
    DOCKER_ENDPOINT_TEST_MESSAGE_MAX_BYTES, DockerEndpointTestCode as ProbeCode,
    DockerEndpointTestData, DockerEndpointTestResultData, DockerEndpointTestStatus as ProbeStatus,
    PACKAGE_INVENTORY_MAX_ITEMS, PACKAGE_VERSION_MAX_BYTES, PackageInfo, PackageInventory,
    PackageListData, PackageListedData, PowerAction, PowerActionData, PowerResultCode,
    PowerResultData, PowerResultStatus,
};

fn power_result(status: PowerResultStatus, code: PowerResultCode) -> PowerResultData {
    PowerResultData {
        request_id: "power-123".into(),
        asset_id: "node-1".into(),
        action: PowerAction::REBOOT,
        status,
        code,
        message: String::new(),
    }
}

/// Risk: the hub records a reboot as done when the agent refused it, or
/// accepts a power request for an action outside reboot and shutdown.
#[test]
fn power_contract_fails_closed() {
    type Status = PowerResultStatus;
    type Code = PowerResultCode;
    let none = Code::default();

    assert!(
        power_result(Status::REJECTED, Code::CAPABILITY_DENIED)
            .validate()
            .is_ok()
    );
    assert!(
        power_result(Status::ACCEPTED, none.clone())
            .validate()
            .is_ok()
    );
    assert!(
        power_result(Status::ACCEPTED, Code::EXECUTION_FAILED)
            .validate()
            .is_err()
    );
    assert!(
        power_result(Status::REJECTED, Code::EXECUTION_FAILED)
            .validate()
            .is_err()
    );
    assert!(
        power_result(Status::UNSUPPORTED, none.clone())
            .validate()
            .is_err()
    );
    assert!(
        power_result(Status::from("succeeded"), none)
            .validate()
            .is_err()
    );

    for action in ["", "restart", "Reboot", "shell"] {
        let request = PowerActionData {
            request_id: "power-123".into(),
            asset_id: "node-1".into(),
            action: action.into(),
        };
        assert!(request.validate().is_err(), "action {action:?} accepted");
    }
}

fn probe_result(status: ProbeStatus, code: ProbeCode) -> DockerEndpointTestResultData {
    DockerEndpointTestResultData {
        request_id: "docker-test-1".into(),
        asset_id: "asset-1".into(),
        endpoint: "unix:///run/docker.sock".into(),
        status,
        code,
        message: String::new(),
    }
}

/// Risk: a Docker endpoint probe runs on unbounded or control-character
/// input, or a failed probe is reported as reachable.
#[test]
fn docker_endpoint_contract_fails_closed() {
    let request = DockerEndpointTestData {
        request_id: "docker-test-1".into(),
        asset_id: "asset-1".into(),
        endpoint: "unix:///run/docker.sock".into(),
    };
    assert!(request.validate().is_ok());
    let broken_requests = [
        DockerEndpointTestData {
            request_id: String::new(),
            ..request.clone()
        },
        DockerEndpointTestData {
            request_id: "r".repeat(129),
            ..request.clone()
        },
        DockerEndpointTestData {
            request_id: "request\n1".into(),
            ..request.clone()
        },
        DockerEndpointTestData {
            asset_id: "a".repeat(257),
            ..request.clone()
        },
        DockerEndpointTestData {
            endpoint: String::new(),
            ..request.clone()
        },
        DockerEndpointTestData {
            endpoint: "unix:///run/docker.sock\0".into(),
            ..request.clone()
        },
        DockerEndpointTestData {
            endpoint: "e".repeat(2049),
            ..request.clone()
        },
    ];
    for (index, broken) in broken_requests.iter().enumerate() {
        assert!(broken.validate().is_err(), "request case {index} accepted");
    }

    let valid = probe_result(ProbeStatus::FAILED, ProbeCode::TIMEOUT);
    assert!(valid.validate().is_ok());
    assert!(
        probe_result(ProbeStatus::REACHABLE, ProbeCode::default())
            .validate()
            .is_ok()
    );
    let broken_results = [
        DockerEndpointTestResultData {
            request_id: String::new(),
            ..valid.clone()
        },
        DockerEndpointTestResultData {
            asset_id: "asset\n1".into(),
            ..valid.clone()
        },
        probe_result("succeeded".into(), ProbeCode::default()),
        probe_result(ProbeStatus::REACHABLE, ProbeCode::UNREACHABLE),
        probe_result(ProbeStatus::REJECTED, ProbeCode::TIMEOUT),
        probe_result(ProbeStatus::FAILED, ProbeCode::BUSY),
        DockerEndpointTestResultData {
            endpoint: String::new(),
            ..valid.clone()
        },
        DockerEndpointTestResultData {
            endpoint: String::new(),
            ..probe_result(ProbeStatus::REJECTED, ProbeCode::ASSET_MISMATCH)
        },
        DockerEndpointTestResultData {
            message: "bad\nmessage".into(),
            ..valid.clone()
        },
        DockerEndpointTestResultData {
            message: "m".repeat(DOCKER_ENDPOINT_TEST_MESSAGE_MAX_BYTES + 1),
            ..valid.clone()
        },
    ];
    for (index, broken) in broken_results.iter().enumerate() {
        assert!(broken.validate().is_err(), "result case {index} accepted");
    }

    // Invalid input cannot be echoed safely, so this one result may omit it.
    let unechoed = DockerEndpointTestResultData {
        endpoint: String::new(),
        ..probe_result(ProbeStatus::REJECTED, ProbeCode::INVALID_REQUEST)
    };
    assert!(unechoed.validate().is_ok());
}

fn package(name: &str, version: &str, available: &str, status: &str) -> PackageInfo {
    PackageInfo {
        name: name.into(),
        version: version.into(),
        available_version: available.into(),
        status: status.into(),
    }
}

fn listed(inventory: PackageInventory, packages: Vec<PackageInfo>) -> PackageListedData {
    PackageListedData {
        request_id: "package-123".into(),
        inventory,
        packages,
        error: String::new(),
    }
}

/// Risk: the hub shows a partial or malformed update list as complete, or
/// takes an older agent's installed list for the list of available updates.
#[test]
fn package_inventory_contract_fails_closed() {
    let upgradable = PackageInventory::UPGRADABLE;
    let curl = package("curl", "8.5.0", "8.6.0", "upgradable");
    assert!(
        listed(upgradable.clone(), vec![curl.clone()])
            .validate()
            .is_ok()
    );
    // An installed package may have no version, as on Windows.
    let unversioned = package("Vendor Utility", "", "", "installed");
    assert!(
        listed(PackageInventory::default(), vec![unversioned])
            .validate()
            .is_ok()
    );

    let long_version = "1".repeat(PACKAGE_VERSION_MAX_BYTES + 1);
    let broken = [
        PackageListedData {
            request_id: String::new(),
            ..listed(upgradable.clone(), vec![])
        },
        listed("updates".into(), vec![]),
        listed(
            upgradable.clone(),
            vec![package("curl", "8.5.0", "", "upgradable")],
        ),
        listed(
            PackageInventory::default(),
            vec![package("curl\n", "8.5.0", "", "installed")],
        ),
        listed(
            PackageInventory::default(),
            vec![package("curl", &long_version, "", "installed")],
        ),
        listed(
            upgradable.clone(),
            vec![curl; PACKAGE_INVENTORY_MAX_ITEMS + 1],
        ),
    ];
    for (index, response) in broken.iter().enumerate() {
        assert!(
            response.validate().is_err(),
            "inventory case {index} accepted"
        );
    }

    for inventory in [
        PackageInventory::default(),
        PackageInventory::INSTALLED,
        upgradable,
    ] {
        let request = PackageListData {
            request_id: "package".into(),
            inventory,
        };
        assert!(request.validate().is_ok());
    }
    let unknown = PackageListData {
        request_id: "package".into(),
        inventory: "updates".into(),
    };
    assert!(unknown.validate().is_err());
}
