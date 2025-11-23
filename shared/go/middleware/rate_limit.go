package middleware

import (
	"fmt"
	"net"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/redis/go-redis/v9"
)

type RateLimiter struct {
	client   *redis.Client
	prefix   string
	maxReq   int
	window   time.Duration
}

// NewRateLimiter creates a new rate limiter
func NewRateLimiter(redisClient *redis.Client, prefix string, maxRequests int, window time.Duration) *RateLimiter {
	return &RateLimiter{
		client:   redisClient,
		prefix:   prefix,
		maxReq:   maxRequests,
		window:   window,
	}
}

// getClientIP extracts the real client IP from headers
func getClientIP(c *gin.Context) string {
	// Check X-Forwarded-For header first (behind reverse proxy)
	if xff := c.GetHeader("X-Forwarded-For"); xff != "" {
		// Take the first IP if multiple are present
		if ip, _, err := net.SplitHostPort(xff); err == nil {
			return ip
		}
		return xff
	}

	// Check X-Real-IP header
	if xri := c.GetHeader("X-Real-IP"); xri != "" {
		return xri
	}

	// Fall back to remote address
	ip, _, _ := net.SplitHostPort(c.Request.RemoteAddr)
	return ip
}

// Middleware returns a rate limiting middleware
func (rl *RateLimiter) Middleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		clientIP := getClientIP(c)
		key := fmt.Sprintf("%s:%s", rl.prefix, clientIP)

		ctx := c.Request.Context()
		count, err := rl.client.Incr(ctx, key).Val(), nil
		
		if err != nil {
			// If Redis fails, allow the request but log
			fmt.Printf("Rate limiter Redis error: %v\n", err)
			c.Next()
			return
		}

		// Set expiration on first request
		if count == 1 {
			rl.client.Expire(ctx, key, rl.window)
		}

		// Add rate limit headers
		c.Header("X-RateLimit-Limit", fmt.Sprintf("%d", rl.maxReq))
		c.Header("X-RateLimit-Remaining", fmt.Sprintf("%d", max(0, rl.maxReq-int(count))))
		c.Header("X-RateLimit-Reset", fmt.Sprintf("%d", time.Now().Add(rl.window).Unix()))

		if count > int64(rl.maxReq) {
			c.JSON(http.StatusTooManyRequests, gin.H{
				"error_type":    "rate_limit_exceeded",
				"error_message": "Too many requests. Please try again later.",
			})
			c.Abort()
			return
		}

		c.Next()
	}
}

// BruteForceProtection tracks failed login attempts
type BruteForceProtection struct {
	client            *redis.Client
	maxAttempts       int
	lockoutDuration   time.Duration
}

func NewBruteForceProtection(redisClient *redis.Client, maxAttempts int, lockoutDuration time.Duration) *BruteForceProtection {
	return &BruteForceProtection{
		client:          redisClient,
		maxAttempts:     maxAttempts,
		lockoutDuration: lockoutDuration,
	}
}

// CheckAttempt checks if the IP is locked out
func (bp *BruteForceProtection) CheckAttempt(clientIP string) (bool, error) {
	key := fmt.Sprintf("failed_attempts:%s", clientIP)
	ctx := time.Background()
	
	count, err := bp.client.Get(ctx, key).Int64()
	if err != nil && err != redis.Nil {
		return false, err
	}

	if count >= int64(bp.maxAttempts) {
		return true, nil // IP is locked out
	}

	return false, nil
}

// RecordFailedAttempt records a failed login attempt
func (bp *BruteForceProtection) RecordFailedAttempt(clientIP string) error {
	key := fmt.Sprintf("failed_attempts:%s", clientIP)
	ctx := time.Background()

	count, err := bp.client.Incr(ctx, key).Val(), nil
	if err != nil {
		return err
	}

	// Set expiration on first attempt
	if count == 1 {
		bp.client.Expire(ctx, key, bp.lockoutDuration)
	}

	return nil
}

// ResetAttempts resets failed attempts for an IP (on successful login)
func (bp *BruteForceProtection) ResetAttempts(clientIP string) error {
	key := fmt.Sprintf("failed_attempts:%s", clientIP)
	ctx := time.Background()
	return bp.client.Del(ctx, key).Err()
}

// IPBlacklist manages blocked IPs
type IPBlacklist struct {
	client   *redis.Client
	duration time.Duration
}

func NewIPBlacklist(redisClient *redis.Client, duration time.Duration) *IPBlacklist {
	return &IPBlacklist{
		client:   redisClient,
		duration: duration,
	}
}

// BlockIP adds an IP to the blacklist
func (ib *IPBlacklist) BlockIP(clientIP string, reason string) error {
	key := fmt.Sprintf("blocked_ip:%s", clientIP)
	ctx := time.Background()
	return ib.client.Set(ctx, key, reason, ib.duration).Err()
}

// IsBlocked checks if an IP is blocked
func (ib *IPBlacklist) IsBlocked(clientIP string) (bool, error) {
	key := fmt.Sprintf("blocked_ip:%s", clientIP)
	ctx := time.Background()
	result, err := ib.client.Exists(ctx, key).Val(), nil
	if err != nil {
		return false, err
	}
	return result == 1, nil
}

// UnblockIP removes an IP from the blacklist
func (ib *IPBlacklist) UnblockIP(clientIP string) error {
	key := fmt.Sprintf("blocked_ip:%s", clientIP)
	ctx := time.Background()
	return ib.client.Del(ctx, key).Err()
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
