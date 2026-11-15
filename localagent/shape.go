package localagent

import (
	"errors"
	"fmt"
)

// CompareStatusShape reports every difference in field names and JSON types
// between two bodies, for example a real agent response and the fixture for
// the same state. Values are not compared. Arrays must have the same length
// and are compared item by item.
func CompareStatusShape(got, want []byte) error {
	gotValue, err := decodeJSON(got)
	if err != nil {
		return fmt.Errorf("got: %w", err)
	}
	wantValue, err := decodeJSON(want)
	if err != nil {
		return fmt.Errorf("want: %w", err)
	}
	return errors.Join(compareShape(gotValue, wantValue, "$")...)
}

func compareShape(got, want any, at string) []error {
	if jsonType(got) != jsonType(want) {
		return []error{fmt.Errorf("%s: got %s, want %s", at, jsonType(got), jsonType(want))}
	}
	var problems []error
	switch wantTyped := want.(type) {
	case map[string]any:
		gotTyped := got.(map[string]any)
		for _, name := range sortedKeys(wantTyped) {
			gotChild, ok := gotTyped[name]
			if !ok {
				problems = append(problems, fmt.Errorf("%s: missing field %q", at, name))
				continue
			}
			problems = append(problems, compareShape(gotChild, wantTyped[name], at+"."+name)...)
		}
		for _, name := range sortedKeys(gotTyped) {
			if _, ok := wantTyped[name]; !ok {
				problems = append(problems, fmt.Errorf("%s: extra field %q", at, name))
			}
		}
	case []any:
		gotTyped := got.([]any)
		if len(gotTyped) != len(wantTyped) {
			return []error{fmt.Errorf("%s: got %d items, want %d", at, len(gotTyped), len(wantTyped))}
		}
		for i := range wantTyped {
			problems = append(problems, compareShape(gotTyped[i], wantTyped[i], fmt.Sprintf("%s[%d]", at, i))...)
		}
	}
	return problems
}
