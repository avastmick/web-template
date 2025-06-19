# Security Guide

This document outlines the security features, best practices, and hardening measures implemented in the web-template application.

## Table of Contents

1. [Security Architecture](#security-architecture)
2. [Authentication and Authorization](#authentication-and-authorization)
3. [Data Protection](#data-protection)
4. [Network Security](#network-security)
5. [Input Validation and Sanitization](#input-validation-and-sanitization)
6. [Security Headers](#security-headers)
7. [Dependency Security](#dependency-security)
8. [Logging and Monitoring](#logging-and-monitoring)
9. [Incident Response](#incident-response)
10. [Security Checklist](#security-checklist)

## Security Architecture

### Defense in Depth

The application implements multiple layers of security:

1. **Network Layer**: HTTPS/TLS, CORS, Rate limiting
2. **Application Layer**: Input validation, Authentication, Authorization
3. **Data Layer**: Encryption at rest, Secure database connections
4. **Infrastructure Layer**: Container security, OS hardening

### Security Principles

- **Least Privilege**: Users and processes have minimal required permissions
- **Zero Trust**: Every request is authenticated and authorized
- **Fail Secure**: System fails to a secure state
- **Security by Design**: Security considerations built into every component

## Authentication and Authorization

### JWT Token Security

**Implementation Details**:
```rust
// JWT Configuration (server/src/services/auth_service.rs)
- Algorithm: HS256 (HMAC-SHA256)
- Token Expiry: 24 hours (configurable)
- Secret Key: Minimum 32 characters, cryptographically random
- Claims: user_id, email, exp, iat
```

**Security Measures**:
- JWT secrets are stored in environment variables only
- Tokens expire automatically
- No sensitive data stored in JWT payload
- Tokens are validated on every protected request

### OAuth 2.0 Implementation

**Google OAuth Security**:
- PKCE (Proof Key for Code Exchange) flow
- State parameter for CSRF protection
- Redirect URI validation
- Scope limitation (openid, email, profile only)

**Invite-Only System**:
- Only users with valid invites can register
- Invites are single-use and time-limited
- Case-insensitive email matching
- Invite validation before user creation

### Password Security

**Local Authentication**:
```rust
// Password hashing using Argon2id
- Algorithm: Argon2id (memory-hard, side-channel resistant)
- Memory cost: 64MB
- Time cost: 3 iterations
- Parallelism: 4 threads
- Salt: Cryptographically random, unique per password
```

**Password Requirements**:
- Minimum 12 characters
- No common password validation
- Passwords never logged or stored in plaintext
- Password reset capability (to be implemented)

## Data Protection

### Encryption at Rest

**Database Security**:
- SQLite database file permissions: 600 (owner read/write only)
- Sensitive fields encrypted before storage (if needed)
- Regular database backups with encryption

**File System Security**:
```bash
# Application files
chmod 755 /opt/your-app/
chmod 600 /opt/your-app/.env

# Database files
chmod 600 /var/lib/your-app/db/production.sqlite3
chown app-user:app-user /var/lib/your-app/db/
```

### Encryption in Transit

**TLS Configuration**:
- TLS 1.2+ required
- Strong cipher suites only
- HSTS headers enabled
- Certificate pinning recommended

### Data Minimization

- Only necessary user data is collected
- PII is limited to email and name (from OAuth)
- No sensitive data in logs
- Data retention policies implemented

## Network Security

### HTTPS Enforcement

**Required Configuration**:
```nginx
# Nginx security configuration
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header X-Frame-Options DENY always;
add_header X-Content-Type-Options nosniff always;
add_header Referrer-Policy strict-origin-when-cross-origin always;
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self';" always;
```

### CORS Configuration

**Rust Backend CORS**:
```rust
// Strict CORS policy
- Allowed Origins: Explicitly configured (no wildcards in production)
- Allowed Methods: GET, POST, OPTIONS only
- Allowed Headers: Content-Type, Authorization
- Credentials: Enabled for authenticated requests
```

### Rate Limiting

**Implementation Recommendations**:
- API rate limiting: 100 requests/minute per IP
- Authentication endpoints: 5 attempts/minute per IP
- OAuth callbacks: 10 requests/minute per IP
- Global rate limiting: 1000 requests/minute per IP

## Input Validation and Sanitization

### Server-Side Validation

**Rust Input Validation**:
```rust
// Using validator crate
#[derive(Validate, Deserialize)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 12, message = "Password must be at least 12 characters"))]
    pub password: String,
}
```

**Validation Rules**:
- Email format validation
- Password strength requirements
- SQL injection prevention (parameterized queries)
- XSS prevention (output encoding)
- Input length limits
- Character set restrictions

### Client-Side Validation

**TypeScript Validation**:
```typescript
// Email validation regex
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

// Password validation
const validatePassword = (password: string): boolean => {
    return password.length >= 12;
};
```

### SQL Injection Prevention

**SQLx Compile-Time Verification**:
```rust
// Parameterized queries prevent SQL injection
sqlx::query!(
    "SELECT * FROM users WHERE email = $1",
    email
)
```

## Security Headers

### HTTP Security Headers

**Implemented Headers**:
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'
```

### Content Security Policy (CSP)

**Recommended CSP**:
```http
Content-Security-Policy:
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: https:;
  connect-src 'self';
  font-src 'self';
  object-src 'none';
  media-src 'self';
  frame-src 'none';
```

## Dependency Security

### Rust Dependencies

**Security Scanning**:
```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

**Dependency Management**:
- Regular dependency updates
- Vulnerability scanning in CI/CD
- Minimal dependency tree
- Trusted crate sources only

### Client Dependencies

**npm/Bun Security**:
```bash
# Audit client dependencies
bun audit

# Check for vulnerabilities
npm audit --audit-level high
```

**Security Practices**:
- Lock file verification
- Automated dependency updates
- Vulnerability alerts enabled
- Package source verification

## Logging and Monitoring

### Security Logging

**Logged Security Events**:
- Authentication attempts (success/failure)
- Authorization failures
- Invalid input attempts
- Rate limiting triggers
- OAuth flow events
- Database connection errors

**Log Security**:
```rust
// Secure logging practices
- No sensitive data in logs (passwords, tokens, PII)
- Structured logging with tracing crate
- Log levels properly configured
- Log rotation and retention policies
```

**Example Log Format**:
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "WARN",
  "module": "auth_handler",
  "event": "login_failed",
  "email": "user@example.com",
  "ip": "192.168.1.100",
  "user_agent": "Mozilla/5.0..."
}
```

### Monitoring and Alerting

**Security Monitoring**:
- Failed authentication attempts
- Unusual access patterns
- Database connection failures
- High error rates
- Certificate expiry warnings

**Alerting Thresholds**:
- > 10 failed logins/minute per IP
- > 100 4xx errors/minute
- > 50 5xx errors/minute
- Certificate expiry < 30 days

## Incident Response

### Security Incident Classification

**Severity Levels**:
1. **Critical**: Data breach, system compromise
2. **High**: Authentication bypass, privilege escalation
3. **Medium**: DoS attack, data exposure
4. **Low**: Failed authentication attempts, minor vulnerabilities

### Response Procedures

**Immediate Response**:
1. Isolate affected systems
2. Preserve evidence
3. Assess scope and impact
4. Implement containment measures
5. Document all actions

**Communication Plan**:
- Internal team notification
- User notification (if data affected)
- Regulatory reporting (if required)
- Public disclosure (if necessary)

### Recovery Procedures

**System Recovery**:
1. Remove threat/vulnerability
2. Restore from clean backups
3. Apply security patches
4. Update credentials
5. Monitor for recurring issues

## Security Checklist

### Development Security

- [ ] Input validation on all user inputs
- [ ] Parameterized database queries
- [ ] Secure password hashing (Argon2id)
- [ ] JWT token validation
- [ ] CORS properly configured
- [ ] Error messages don't leak information
- [ ] Secrets not in source code
- [ ] Dependencies regularly updated

### Infrastructure Security

- [ ] HTTPS/TLS properly configured
- [ ] Security headers implemented
- [ ] Rate limiting configured
- [ ] Database file permissions set
- [ ] Application runs as non-root user
- [ ] Firewall rules configured
- [ ] Log monitoring enabled
- [ ] Backup encryption enabled

### OAuth Security

- [ ] Production OAuth credentials configured
- [ ] Redirect URIs properly validated
- [ ] State parameter implemented
- [ ] Scope limitations enforced
- [ ] Token validation working
- [ ] Error handling secure
- [ ] User consent screen completed

### Deployment Security

- [ ] Environment variables secured
- [ ] Database backups encrypted
- [ ] Monitoring and alerting configured
- [ ] Incident response plan ready
- [ ] Security testing completed
- [ ] Penetration testing performed
- [ ] Compliance requirements met

### Regular Security Maintenance

- [ ] Weekly dependency audits
- [ ] Monthly security reviews
- [ ] Quarterly penetration testing
- [ ] Annual security assessment
- [ ] Continuous monitoring
- [ ] Regular backup testing
- [ ] Certificate renewal tracking

## Security Testing

### Automated Testing

**Security Test Suite**:
```bash
# Run security tests
just test-security

# Dependency audit
cargo audit
bun audit

# SAST (Static Application Security Testing)
cargo clippy -- -D warnings

# Secret scanning
git secrets --scan
```

### Manual Testing

**Security Testing Checklist**:
- [ ] SQL injection testing
- [ ] XSS vulnerability testing
- [ ] Authentication bypass attempts
- [ ] Authorization testing
- [ ] Session management testing
- [ ] CSRF protection testing
- [ ] Input validation testing
- [ ] Error handling testing

### Penetration Testing

**Recommended Testing Areas**:
- Authentication and authorization
- Input validation and sanitization
- Session management
- Business logic vulnerabilities
- Configuration and deployment
- Information disclosure

## Compliance and Standards

### Standards Compliance

- **OWASP Top 10**: All items addressed
- **CWE/SANS Top 25**: Most dangerous weaknesses mitigated
- **NIST Cybersecurity Framework**: Core functions implemented

### Privacy Compliance

- **GDPR Considerations**:
  - Data minimization
  - User consent (OAuth)
  - Right to deletion (to be implemented)
  - Data portability (to be implemented)

## Contact and Reporting

### Security Contact

For security issues or questions:
- Email: security@yourdomain.com
- PGP Key: [Include public key or link]

### Responsible Disclosure

We encourage responsible disclosure of security vulnerabilities:
1. Report privately to security contact
2. Allow reasonable time for response
3. Work with us on disclosure timeline
4. Credit will be given for valid reports

### Bug Bounty

Consider implementing a bug bounty program for:
- Authentication and authorization bypasses
- Data exposure vulnerabilities
- Remote code execution
- SQL injection
- XSS vulnerabilities

This security guide should be reviewed and updated regularly as the application evolves and new threats emerge.
