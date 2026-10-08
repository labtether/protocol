package protocol

// AlertNotifyData is sent from the hub to the agent when an alert fires or resolves.
type AlertNotifyData struct {
	ID        string `json:"id"`
	Severity  string `json:"severity"` // critical, high, medium, low
	Title     string `json:"title"`
	Summary   string `json:"summary"`
	State     string `json:"state"` // firing, resolved
	Timestamp string `json:"timestamp"`
}

// EnrollmentApprovedData is sent from hub to agent when enrollment is approved.
type EnrollmentApprovedData struct {
	Token   string `json:"token"`
	AssetID string `json:"asset_id"`
}

// EnrollmentChallengeData is sent from hub to agent while pending enrollment.
// The agent signs the challenge and returns EnrollmentProofData.
type EnrollmentChallengeData struct {
	ConnectionID string `json:"connection_id"`
	Nonce        string `json:"nonce"`
	ExpiresAt    string `json:"expires_at,omitempty"`
}

// EnrollmentProofData is sent from agent to hub to prove possession of the
// device private key for pending enrollment verification.
type EnrollmentProofData struct {
	ConnectionID string `json:"connection_id"`
	Nonce        string `json:"nonce"`
	KeyAlgorithm string `json:"key_algorithm"`
	PublicKey    string `json:"public_key"`  // base64 raw public key bytes
	Fingerprint  string `json:"fingerprint"` // human-friendly fingerprint
	Signature    string `json:"signature"`   // base64 signature over canonical challenge payload
}

// EnrollmentRejectedData is sent from hub to agent when enrollment is rejected.
type EnrollmentRejectedData struct {
	Reason string `json:"reason,omitempty"`
}
