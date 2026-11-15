package localagent

import (
	"encoding/json"
	"fmt"
	"regexp"
	"sort"
	"strings"
	"unicode/utf8"
)

// schemaChecker applies the part of JSON Schema 2020-12 that
// status.v1.schema.json uses. It refuses a schema with any other keyword, so
// the schema cannot grow a rule that this check would silently skip.
type schemaChecker struct {
	root     map[string]any
	patterns map[string]*regexp.Regexp
}

var supportedKeywords = map[string]bool{
	"$schema": true, "$id": true, "$defs": true, "$ref": true,
	"title": true, "description": true,
	"type": true, "enum": true, "required": true, "properties": true,
	"additionalProperties": true, "items": true, "maxItems": true,
	"pattern": true, "minLength": true, "maxLength": true,
	"minimum": true, "maximum": true,
}

func newSchemaChecker(root map[string]any) (*schemaChecker, error) {
	checker := &schemaChecker{root: root, patterns: map[string]*regexp.Regexp{}}
	if err := checker.compile(root, "#"); err != nil {
		return nil, err
	}
	return checker, nil
}

// compile checks every keyword and pre-compiles every pattern.
func (c *schemaChecker) compile(node map[string]any, at string) error {
	for _, key := range sortedKeys(node) {
		value := node[key]
		if !supportedKeywords[key] {
			return fmt.Errorf("schema %s: unsupported keyword %q", at, key)
		}
		switch key {
		case "pattern":
			text, _ := value.(string)
			compiled, err := regexp.Compile(text)
			if err != nil {
				return fmt.Errorf("schema %s: pattern: %w", at, err)
			}
			c.patterns[text] = compiled
		case "type":
			switch value {
			case "object", "array", "string", "number", "boolean":
			default:
				return fmt.Errorf("schema %s: unsupported type %v", at, value)
			}
		case "additionalProperties":
			if value != false {
				return fmt.Errorf("schema %s: additionalProperties must be false", at)
			}
		case "$ref":
			if _, err := c.resolve(value); err != nil {
				return fmt.Errorf("schema %s: %w", at, err)
			}
		case "properties", "$defs":
			children, ok := value.(map[string]any)
			if !ok {
				return fmt.Errorf("schema %s/%s: not an object", at, key)
			}
			for _, name := range sortedKeys(children) {
				child, ok := children[name].(map[string]any)
				if !ok {
					return fmt.Errorf("schema %s/%s/%s: not an object", at, key, name)
				}
				if err := c.compile(child, at+"/"+key+"/"+name); err != nil {
					return err
				}
			}
		case "items":
			child, ok := value.(map[string]any)
			if !ok {
				return fmt.Errorf("schema %s/items: not an object", at)
			}
			if err := c.compile(child, at+"/items"); err != nil {
				return err
			}
		}
	}
	return nil
}

func (c *schemaChecker) resolve(ref any) (map[string]any, error) {
	text, _ := ref.(string)
	name, ok := strings.CutPrefix(text, "#/$defs/")
	if !ok {
		return nil, fmt.Errorf("unsupported $ref %q", text)
	}
	defs, _ := c.root["$defs"].(map[string]any)
	target, ok := defs[name].(map[string]any)
	if !ok {
		return nil, fmt.Errorf("unknown $ref %q", text)
	}
	return target, nil
}

func (c *schemaChecker) validate(value any) []error {
	return c.check(c.root, value, "$")
}

func (c *schemaChecker) check(node map[string]any, value any, at string) []error {
	if ref, ok := node["$ref"]; ok {
		target, _ := c.resolve(ref)
		return c.check(target, value, at)
	}
	if want, ok := node["type"].(string); ok && !hasType(value, want) {
		return []error{fmt.Errorf("%s: want %s, got %s", at, want, jsonType(value))}
	}
	var problems []error
	if allowed, ok := node["enum"].([]any); ok && !inList(allowed, value) {
		problems = append(problems, fmt.Errorf("%s: %s is not one of %s", at, jsonText(value), jsonText(allowed)))
	}
	switch typed := value.(type) {
	case string:
		problems = append(problems, c.checkString(node, typed, at)...)
	case json.Number:
		problems = append(problems, checkNumber(node, typed, at)...)
	case []any:
		problems = append(problems, c.checkArray(node, typed, at)...)
	case map[string]any:
		problems = append(problems, c.checkObject(node, typed, at)...)
	}
	return problems
}

func (c *schemaChecker) checkString(node map[string]any, text, at string) []error {
	var problems []error
	length := float64(utf8.RuneCountInString(text))
	if limit, ok := node["minLength"].(json.Number); ok && length < mustFloat(limit) {
		problems = append(problems, fmt.Errorf("%s: shorter than %s characters", at, limit))
	}
	if limit, ok := node["maxLength"].(json.Number); ok && length > mustFloat(limit) {
		problems = append(problems, fmt.Errorf("%s: longer than %s characters", at, limit))
	}
	if pattern, ok := node["pattern"].(string); ok && !c.patterns[pattern].MatchString(text) {
		problems = append(problems, fmt.Errorf("%s: %q does not match %s", at, text, pattern))
	}
	return problems
}

func checkNumber(node map[string]any, number json.Number, at string) []error {
	value, err := number.Float64()
	if err != nil {
		return []error{fmt.Errorf("%s: %w", at, err)}
	}
	var problems []error
	if limit, ok := node["minimum"].(json.Number); ok && value < mustFloat(limit) {
		problems = append(problems, fmt.Errorf("%s: %s is below %s", at, number, limit))
	}
	if limit, ok := node["maximum"].(json.Number); ok && value > mustFloat(limit) {
		problems = append(problems, fmt.Errorf("%s: %s is above %s", at, number, limit))
	}
	return problems
}

func (c *schemaChecker) checkArray(node map[string]any, items []any, at string) []error {
	var problems []error
	if limit, ok := node["maxItems"].(json.Number); ok && float64(len(items)) > mustFloat(limit) {
		problems = append(problems, fmt.Errorf("%s: more than %s items", at, limit))
	}
	if child, ok := node["items"].(map[string]any); ok {
		for i, item := range items {
			problems = append(problems, c.check(child, item, fmt.Sprintf("%s[%d]", at, i))...)
		}
	}
	return problems
}

func (c *schemaChecker) checkObject(node map[string]any, object map[string]any, at string) []error {
	var problems []error
	required, _ := node["required"].([]any)
	for _, name := range required {
		if _, ok := object[name.(string)]; !ok {
			problems = append(problems, fmt.Errorf("%s: missing required field %q", at, name))
		}
	}
	properties, _ := node["properties"].(map[string]any)
	for _, name := range sortedKeys(object) {
		child, known := properties[name].(map[string]any)
		if !known {
			if node["additionalProperties"] == false {
				problems = append(problems, fmt.Errorf("%s: unknown field %q", at, name))
			}
			continue
		}
		problems = append(problems, c.check(child, object[name], at+"."+name)...)
	}
	return problems
}

func hasType(value any, want string) bool {
	return jsonType(value) == want
}

func jsonType(value any) string {
	switch value.(type) {
	case map[string]any:
		return "object"
	case []any:
		return "array"
	case string:
		return "string"
	case json.Number:
		return "number"
	case bool:
		return "boolean"
	case nil:
		return "null"
	}
	return fmt.Sprintf("%T", value)
}

func inList(list []any, value any) bool {
	for _, item := range list {
		if item == value {
			return true
		}
	}
	return false
}

func jsonText(value any) string {
	text, err := json.Marshal(value)
	if err != nil {
		return fmt.Sprint(value)
	}
	return string(text)
}

func mustFloat(number json.Number) float64 {
	value, _ := number.Float64()
	return value
}

func sortedKeys(object map[string]any) []string {
	keys := make([]string, 0, len(object))
	for key := range object {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys
}
