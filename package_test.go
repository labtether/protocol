package protocol

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestPackageInventoryClosedValues(t *testing.T) {
	for _, inventory := range []PackageInventory{"", PackageInventoryInstalled, PackageInventoryUpgradable} {
		if !inventory.Valid() {
			t.Fatalf("expected inventory %q to be valid", inventory)
		}
	}
	for _, inventory := range []PackageInventory{"updates", "UPGRADABLE", "all"} {
		if inventory.Valid() {
			t.Fatalf("expected inventory %q to be rejected", inventory)
		}
	}
}

func TestPackageUpgradableWireRoundTrip(t *testing.T) {
	want := PackageListedData{
		RequestID: "package-123",
		Inventory: PackageInventoryUpgradable,
		Packages: []PackageInfo{{
			Name:             "curl",
			Version:          "8.5.0",
			AvailableVersion: "8.6.0",
			Status:           "upgradable",
		}},
	}
	raw, err := json.Marshal(want)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}

	var got PackageListedData
	if err := json.Unmarshal(raw, &got); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if got.RequestID != want.RequestID || got.Inventory != want.Inventory || len(got.Packages) != 1 || got.Packages[0] != want.Packages[0] {
		t.Fatalf("round-trip mismatch: got %+v want %+v", got, want)
	}
}

func TestPackageInstalledWireRemainsBackwardCompatible(t *testing.T) {
	legacy := []byte(`{"request_id":"package-legacy","packages":[{"name":"jq","version":"1.7","status":"installed"}]}`)
	var got PackageListedData
	if err := json.Unmarshal(legacy, &got); err != nil {
		t.Fatalf("unmarshal legacy response: %v", err)
	}
	if got.Inventory != "" || got.Packages[0].AvailableVersion != "" {
		t.Fatalf("legacy response changed semantics: %+v", got)
	}

	raw, err := json.Marshal(PackageListData{RequestID: "package-legacy"})
	if err != nil {
		t.Fatalf("marshal legacy request: %v", err)
	}
	if string(raw) != `{"request_id":"package-legacy"}` {
		t.Fatalf("legacy request gained fields: %s", raw)
	}
}

func TestPackageActionNamesDocumentCanonicalUpgradeAndAlias(t *testing.T) {
	if PackageActionInstall != "install" || PackageActionRemove != "remove" || PackageActionUpgrade != "upgrade" {
		t.Fatal("canonical package action names changed")
	}
	if PackageActionUpdate != "update" {
		t.Fatal("public update compatibility alias changed")
	}
}

func TestPackageInventoryValidationRejectsMalformedAndOversized(t *testing.T) {
	valid := PackageListedData{
		RequestID: "package-123",
		Inventory: PackageInventoryUpgradable,
		Packages: []PackageInfo{{
			Name:             "curl",
			Version:          "8.5.0",
			AvailableVersion: "8.6.0",
			Status:           "upgradable",
		}},
	}
	if err := valid.Validate(); err != nil {
		t.Fatalf("valid inventory rejected: %v", err)
	}

	tests := []PackageListedData{
		{RequestID: "", Inventory: PackageInventoryUpgradable},
		{RequestID: "package-123", Inventory: "updates"},
		{RequestID: "package-123", Inventory: PackageInventoryUpgradable, Packages: []PackageInfo{{Name: "curl", Version: "8.5.0", Status: "upgradable"}}},
		{RequestID: "package-123", Packages: []PackageInfo{{Name: "curl\n", Version: "8.5.0", Status: "installed"}}},
		{RequestID: "package-123", Packages: []PackageInfo{{Name: "curl", Version: strings.Repeat("1", PackageVersionMaxBytes+1), Status: "installed"}}},
	}
	for index, response := range tests {
		if err := response.Validate(); err == nil {
			t.Fatalf("case %d: malformed inventory accepted", index)
		}
	}

	oversized := valid
	oversized.Packages = make([]PackageInfo, PackageInventoryMaxItems+1)
	if err := oversized.Validate(); err == nil {
		t.Fatal("oversized inventory accepted")
	}
}

func TestPackageInventoryRequestValidation(t *testing.T) {
	for _, request := range []PackageListData{
		{RequestID: "package-installed"},
		{RequestID: "package-installed", Inventory: PackageInventoryInstalled},
		{RequestID: "package-upgradable", Inventory: PackageInventoryUpgradable},
	} {
		if err := request.Validate(); err != nil {
			t.Fatalf("valid request rejected: %v", err)
		}
	}
	if err := (PackageListData{RequestID: "package", Inventory: "updates"}).Validate(); err == nil {
		t.Fatal("unknown inventory request accepted")
	}
}

func TestInstalledPackageValidationAllowsMissingRegistryVersion(t *testing.T) {
	response := PackageListedData{
		RequestID: "package-windows-registry",
		Packages:  []PackageInfo{{Name: "Vendor Utility", Status: "installed"}},
	}
	if err := response.Validate(); err != nil {
		t.Fatalf("installed package without registry version rejected: %v", err)
	}
}
