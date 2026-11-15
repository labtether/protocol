// Package localagent holds the contract between the LabTether agent and the
// desktop apps that start it: the shape of GET /agent/status on the agent's
// local API, its closed code sets and one example body per agent state.
//
// The files are embedded so that agent tests can check a real response with
// ValidateStatus and CompareStatusShape. The package uses only the standard
// library.
package localagent

import (
	"bytes"
	"embed"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"path"
	"sort"
	"strings"
)

// Files holds the contract files at their paths in this folder.
//
//go:embed status.v1.schema.json codes.v1.json fixtures/status/*.json
var Files embed.FS

const (
	// StatusSchemaPath is the JSON Schema for the /agent/status body.
	StatusSchemaPath = "status.v1.schema.json"
	// CodesPath holds the closed sets and the rules between fields.
	CodesPath = "codes.v1.json"
	// StatusFixtureDir holds one /agent/status body per agent state.
	StatusFixtureDir = "fixtures/status"
)

// StatusFixtureNames returns the fixture names (file names without .json).
func StatusFixtureNames() ([]string, error) {
	entries, err := fs.ReadDir(Files, StatusFixtureDir)
	if err != nil {
		return nil, err
	}
	names := make([]string, 0, len(entries))
	for _, entry := range entries {
		if name, ok := strings.CutSuffix(entry.Name(), ".json"); ok && !entry.IsDir() {
			names = append(names, name)
		}
	}
	sort.Strings(names)
	return names, nil
}

// StatusFixture returns the body of one status fixture.
func StatusFixture(name string) ([]byte, error) {
	return Files.ReadFile(path.Join(StatusFixtureDir, name+".json"))
}

// ValidateStatus checks a /agent/status body against status.v1.schema.json
// and the rules in codes.v1.json. It returns every problem it finds.
func ValidateStatus(body []byte) error {
	schema, err := loadStatusSchema()
	if err != nil {
		return err
	}
	codes, err := LoadCodes()
	if err != nil {
		return err
	}
	value, err := decodeJSON(body)
	if err != nil {
		return fmt.Errorf("status body: %w", err)
	}
	problems := schema.validate(value)
	problems = append(problems, codes.check(value)...)
	return errors.Join(problems...)
}

func loadStatusSchema() (*schemaChecker, error) {
	raw, err := Files.ReadFile(StatusSchemaPath)
	if err != nil {
		return nil, err
	}
	value, err := decodeJSON(raw)
	if err != nil {
		return nil, fmt.Errorf("%s: %w", StatusSchemaPath, err)
	}
	root, ok := value.(map[string]any)
	if !ok {
		return nil, fmt.Errorf("%s: not an object", StatusSchemaPath)
	}
	return newSchemaChecker(root)
}

// decodeJSON reads one JSON value, keeping numbers as json.Number so that
// the type checks see what was sent.
func decodeJSON(raw []byte) (any, error) {
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		return nil, err
	}
	if decoder.More() {
		return nil, errors.New("more than one JSON value")
	}
	return value, nil
}
