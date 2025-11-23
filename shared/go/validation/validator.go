package validation

import (
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

// Validator provides input validation utilities
type Validator struct{}

// NewValidator creates a new validator
func NewValidator() *Validator {
	return &Validator{}
}

// SanitizeString removes potentially dangerous characters
func (v *Validator) SanitizeString(input string) string {
	// Remove null bytes
	input = strings.ReplaceAll(input, "\x00", "")
	
	// Remove control characters except tab and newline
	input = strings.Map(func(r rune) rune {
		if r < 32 && r != '\t' && r != '\n' && r != '\r' {
			return -1 // Remove
		}
		return r
	}, input)

	// Trim whitespace
	return strings.TrimSpace(input)
}

// ValidateEmail validates an email address
func (v *Validator) ValidateEmail(email string) bool {
	email = v.SanitizeString(email)
	
	// Simple email regex - in production use more robust validation
	emailRegex := regexp.MustCompile(`^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`)
	return emailRegex.MatchString(email) && len(email) <= 254
}

// ValidatePhoneNumber validates a phone number
func (v *Validator) ValidatePhoneNumber(phone string) bool {
	phone = v.SanitizeString(phone)
	
	// Remove common formatting characters
	phone = strings.NewReplacer(
		"+", "",
		"-", "",
		" ", "",
		"(", "",
		")", "",
	).Replace(phone)

	// Must be digits only and 7-15 characters (E.164 standard)
	if len(phone) < 7 || len(phone) > 15 {
		return false
	}

	return regexp.MustCompile(`^\d+$`).MatchString(phone)
}

// ValidateUsername validates a username
func (v *Validator) ValidateUsername(username string) bool {
	username = v.SanitizeString(username)
	
	if len(username) < 3 || len(username) > 32 {
		return false
	}

	// Only alphanumeric, underscore, and hyphen
	usernameRegex := regexp.MustCompile(`^[a-zA-Z0-9_-]+$`)
	return usernameRegex.MatchString(username)
}

// ValidatePassword validates password strength
func (v *Validator) ValidatePassword(password string) (bool, []string) {
	errors := []string{}

	if len(password) < 12 {
		errors = append(errors, "Password must be at least 12 characters long")
	}

	if len(password) > 128 {
		errors = append(errors, "Password must be at most 128 characters long")
	}

	hasUppercase := false
	hasLowercase := false
	hasDigit := false
	hasSpecial := false

	for _, char := range password {
		switch {
		case unicode.IsUpper(char):
			hasUppercase = true
		case unicode.IsLower(char):
			hasLowercase = true
		case unicode.IsDigit(char):
			hasDigit = true
		case !unicode.IsLetter(char) && !unicode.IsDigit(char):
			hasSpecial = true
		}
	}

	if !hasUppercase {
		errors = append(errors, "Password must contain at least one uppercase letter")
	}
	if !hasLowercase {
		errors = append(errors, "Password must contain at least one lowercase letter")
	}
	if !hasDigit {
		errors = append(errors, "Password must contain at least one digit")
	}
	if !hasSpecial {
		errors = append(errors, "Password must contain at least one special character")
	}

	return len(errors) == 0, errors
}

// ValidateUUID validates UUID format
func (v *Validator) ValidateUUID(uid string) bool {
	uuidRegex := regexp.MustCompile(`^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$`)
	return uuidRegex.MatchString(strings.ToLower(uid))
}

// ValidateURL validates URL format
func (v *Validator) ValidateURL(urlStr string) bool {
	urlStr = v.SanitizeString(urlStr)
	
	// Basic URL validation - starts with http/https
	if !strings.HasPrefix(urlStr, "http://") && !strings.HasPrefix(urlStr, "https://") {
		return false
	}

	if len(urlStr) > 2048 {
		return false
	}

	// Check for invalid characters
	return !strings.ContainsAny(urlStr, "\x00\r\n")
}

// ValidateJSON checks if a string is valid JSON (basic check)
func (v *Validator) ValidateJSON(jsonStr string) bool {
	jsonStr = v.SanitizeString(jsonStr)
	if len(jsonStr) < 2 {
		return false
	}

	// Must start with { or [
	return (strings.HasPrefix(jsonStr, "{") || strings.HasPrefix(jsonStr, "[")) &&
		(strings.HasSuffix(jsonStr, "}") || strings.HasSuffix(jsonStr, "]"))
}

// ValidateInputLength checks if input is within allowed length
func (v *Validator) ValidateInputLength(input string, minLen, maxLen int) bool {
	input = v.SanitizeString(input)
	return len(input) >= minLen && len(input) <= maxLen
}

// SanitizeSQL removes potential SQL injection patterns
func (v *Validator) SanitizeSQL(input string) string {
	input = v.SanitizeString(input)
	
	// This is for informational purposes - ALWAYS use parameterized queries
	dangerous := map[string]string{
		"'":       "''",
		"\"":      "\"\"",
		"--":      "",
		"/*":      "",
		"*/":      "",
		"xp_":     "",
		"sp_":     "",
		"DROP":    "",
		"INSERT":  "",
		"UPDATE":  "",
		"DELETE":  "",
		"EXEC":    "",
		"EXECUTE": "",
	}

	result := input
	for old, new := range dangerous {
		result = strings.ReplaceAll(result, old, new)
	}

	return result
}

// ValidateBase64 validates base64 string
func (v *Validator) ValidateBase64(input string) bool {
	input = v.SanitizeString(input)
	base64Regex := regexp.MustCompile(`^[A-Za-z0-9+/]*={0,2}$`)
	
	if len(input)%4 != 0 {
		return false
	}

	return base64Regex.MatchString(input)
}

// ValidateIPAddress validates IP address format
func (v *Validator) ValidateIPAddress(ip string) bool {
	ip = v.SanitizeString(ip)
	
	parts := strings.Split(ip, ".")
	if len(parts) != 4 {
		return false
	}

	for _, part := range parts {
		if part == "" {
			return false
		}
		var num int
		if _, err := fmt.Sscanf(part, "%d", &num); err != nil || num < 0 || num > 255 {
			return false
		}
	}

	return true
}

// Sanitizers for common use cases

// SanitizeDisplayName sanitizes user display names
func (v *Validator) SanitizeDisplayName(name string) (string, bool) {
	name = v.SanitizeString(name)
	
	if !v.ValidateInputLength(name, 1, 100) {
		return "", false
	}

	// Allow letters, numbers, spaces, hyphens, apostrophes
	displayNameRegex := regexp.MustCompile(`^[a-zA-Z0-9\s\-']+$`)
	if !displayNameRegex.MatchString(name) {
		return "", false
	}

	return name, true
}

// SanitizeMessageContent sanitizes message content
func (v *Validator) SanitizeMessageContent(content string) (string, bool) {
	content = v.SanitizeString(content)
	
	if len(content) == 0 || len(content) > 5000 {
		return "", false
	}

	// Messages can contain most characters but not null bytes or other control chars
	return content, true
}
