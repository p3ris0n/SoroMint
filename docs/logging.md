# Logging with Winston

## Overview

SoroMint implements comprehensive structured logging using Winston, a popular and flexible logging library for Node.js. This logging system provides multi-channel output (console and file), correlation ID support for request tracing, and structured log formats for easy parsing and analysis.

## Architecture

```
┌─────────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Incoming Request   │────▶│  Correlation ID  │────▶│  HTTP Logger    │
│                     │     │  Middleware      │     │  Middleware     │
└─────────────────────┘     └──────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  File Transport     │◀────│  Winston Logger  │◀────│  Route Handler  │
│  (logs/server.log)  │     │  (Multi-level)   │     │  (with logging) │
└─────────────────────┘     └──────────────────┘     └─────────────────┘
         │
         ▼
┌─────────────────────┐
│  Console Transport  │
│  (Development)      │
└─────────────────────┘
```

## Components

### 1. Winston Logger

The main logging utility configured with multiple transports and structured formatting.

**Location:** `server/utils/logger.js`

### 2. Correlation ID Middleware

Automatically assigns or extracts a unique correlation ID to each request for end-to-end tracing.

### 3. HTTP Logger Middleware

Logs all HTTP requests with method, URL, status code, duration, and client information.

## Log Levels

| Level | Description | When Used |
|-------|-------------|-----------|
| `error` | Critical errors | Server errors (5xx), exceptions |
| `warn` | Warning messages | Client errors (4xx), recoverable issues |
| `info` | Informational | Successful operations, startup/shutdown |
| `http` | HTTP requests | All HTTP requests (2xx, 3xx) |
| `debug` | Debug information | Detailed debugging info, route registration |

## Log Output Channels

### Console Transport

- **Purpose:** Development and debugging
- **Format:** Human-readable with colors
- **Level:** Debug (development), Info (production)

Example console output:
```
2024-03-23 10:30:45 info: [abc-123-def] Server starting port=5000 network=Futurenet
2024-03-23 10:30:46 info: [xyz-789-uvw] HTTP Request method=GET url=/api/tokens statusCode=200 durationMs=45
```

### File Transport

- **Purpose:** Production logging and archival
- **Format:** Structured JSON
- **Location:** `logs/server.log`
- **Rotation:** Daily rotation, 30 days retention, 20MB max file size

Example JSON log entry:
```json
{
  "level": "info",
  "message": "Token created successfully",
  "timestamp": "2024-03-23 10:30:45.123",
  "correlationId": "abc-123-def",
  "service": "soromint-server",
  "environment": "development",
  "tokenId": "65f1234567890abcdef12345"
}
```

## Usage

### Basic Logging

```javascript
const { logger } = require('./utils/logger');

// Info level
logger.info('Operation completed', { userId: '123', action: 'login' });

// Warning level
logger.warn('Rate limit approaching', { current: 90, limit: 100 });

// Error level
logger.error('Database connection failed', { error: err.message, stack: err.stack });

// Debug level
logger.debug('Processing request', { payload: req.body });
```

### Correlation ID Middleware

Automatically attached to all requests:

```javascript
const { correlationIdMiddleware, httpLoggerMiddleware } = require('./utils/logger');

// In Express app setup
app.use(correlationIdMiddleware);
app.use(httpLoggerMiddleware);

// In route handlers
app.get('/api/tokens', (req, res) => {
  logger.info('Fetching tokens', { correlationId: req.correlationId });
  // ...
});
```

### HTTP Request Logging

All HTTP requests are automatically logged with:

- Method (GET, POST, etc.)
- URL path
- Status code
- Response duration (ms)
- Client IP address
- User agent
- Correlation ID

```javascript
// Automatic - no code needed in routes
app.use(httpLoggerMiddleware);
```

### Helper Functions

#### logStartupInfo

Logs server startup configuration:

```javascript
const { logStartupInfo } = require('./utils/logger');

app.listen(PORT, () => {
  logStartupInfo(PORT, process.env.NETWORK_PASSPHRASE);
});
```

#### logShutdownInfo

Logs graceful shutdown events:

```javascript
const { logShutdownInfo } = require('./utils/logger');

process.on('SIGTERM', () => {
  logShutdownInfo('SIGTERM');
  // Cleanup code...
});
```

#### logDatabaseConnection

Logs MongoDB connection events:

```javascript
const { logDatabaseConnection } = require('./utils/logger');

mongoose.connect(uri)
  .then(() => logDatabaseConnection(true))
  .catch(err => logDatabaseConnection(false, err));
```

#### logRouteRegistration

Logs route registration (debug level):

```javascript
const { logRouteRegistration } = require('./utils/logger');

logRouteRegistration('GET', '/api/tokens');
```

## Log Format

### Console Format (Development)

```
TIMESTAMP LEVEL: [CORRELATION_ID] MESSAGE KEY=VALUE KEY=VALUE
```

Example:
```
2024-03-23 10:30:45 info: [abc-123] Token created tokenId=65f1234567890abcdef12345
```

### JSON Format (File)

```json
{
  "level": "info",
  "message": "Token created",
  "timestamp": "2024-03-23 10:30:45.123",
  "correlationId": "abc-123",
  "service": "soromint-server",
  "environment": "development",
  "tokenId": "65f1234567890abcdef12345"
}
```

## Correlation IDs

### What are Correlation IDs?

Correlation IDs are unique identifiers assigned to each request that allow you to trace the request's journey through the entire system.

### How They Work

1. **Incoming Request:** Client sends request (optionally with `X-Correlation-ID` header)
2. **ID Assignment:** If no ID provided, server generates a new UUID v4
3. **Propagation:** ID attached to request object and response header
4. **Logging:** All logs for that request include the correlation ID
5. **Response:** Client receives response with `X-Correlation-ID` header

### Example Flow

```
Client Request:
  GET /api/tokens
  X-Correlation-ID: abc-123 (optional)

Server Processing:
  [abc-123] Fetching tokens for owner
  [abc-123] Database query executed
  [abc-123] Tokens returned

Client Response:
  200 OK
  X-Correlation-ID: abc-123
```

### Manual Correlation ID

Clients can provide their own correlation ID:

```bash
curl -H "X-Correlation-ID: my-custom-id-123" http://localhost:5000/api/tokens
```

## Integration with Error Handling

The error handler middleware automatically logs errors with appropriate levels:

```javascript
const { errorHandler } = require('./middleware/error-handler');

// 5xx errors -> logger.error
// 4xx errors -> logger.warn
// Other errors -> logger.info

app.use(errorHandler);
```

Error logs include:
- Error message
- Error code
- HTTP status code
- Request path and method
- Correlation ID
- Stack trace (if available)

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `NODE_ENV` | Environment mode | `development` |
| `LOG_LEVEL` | Minimum log level | `debug` (dev), `info` (prod) |

### Log Level Hierarchy

```
error > warn > info > http > debug
```

Setting `LOG_LEVEL=warn` will log `warn` and `error` only.

## Log File Management

### Location

Logs are stored in `logs/server.log`

### Rotation Policy

- **Frequency:** Daily
- **Retention:** 30 days
- **Max File Size:** 20MB
- **Format:** `server.log.YYYY-MM-DD`

### Archived Logs

Old logs are automatically rotated and compressed:
```
logs/
  server.log          # Current day's log
  server.log.2024-03-22
  server.log.2024-03-21
  ...
```

## Best Practices

1. **Always include correlation IDs** in logs for request tracing
2. **Use appropriate log levels** for different types of events
3. **Include context** (user IDs, operation names) in log metadata
4. **Never log sensitive data** (passwords, tokens, PII)
5. **Use structured logging** (objects) instead of string concatenation
6. **Log both success and failure** cases for complete visibility

## Anti-Patterns to Avoid

```javascript
// ❌ Don't log sensitive data
logger.info('User login', { password: user.password });

// ❌ Don't log without context
logger.info('Done');

// ❌ Don't concatenate strings for structured data
logger.info('User ' + userId + ' did ' + action);

// ✅ Do use structured logging
logger.info('User action', { userId, action, timestamp: Date.now() });
```

## Monitoring and Analysis

### Searching Logs

Find all logs for a specific request:
```bash
grep "abc-123-def" logs/server.log
```

### Finding Errors

Find all error logs:
```bash
grep '"level":"error"' logs/server.log
```

### Analyzing Response Times

Extract HTTP request durations:
```bash
grep '"level":"http"' logs/server.log | jq '.durationMs'
```

## Testing

### Unit Tests

Run logger tests:
```bash
cd server
npm test -- tests/utils/logger.test.js
```

### Manual Testing

1. **Start the server:**
```bash
npm run dev
```

2. **Make a request:**
```bash
curl http://localhost:5000/api/status
```

3. **Check console output:**
```
2024-03-23 10:30:45 info: [correlation-id] HTTP Request method=GET url=/api/status statusCode=200
```

4. **Check log file:**
```bash
cat logs/server.log
```

## Troubleshooting

### Logs Not Appearing in File

1. Check `logs/` directory exists
2. Verify write permissions
3. Check disk space

### Too Many Logs

1. Increase `LOG_LEVEL` to reduce verbosity
2. Adjust rotation settings in `winston-daily-rotate-file`
3. Implement log sampling for high-traffic endpoints

### Missing Correlation IDs

1. Ensure `correlationIdMiddleware` is registered before route handlers
2. Check middleware order in Express app

## Security Considerations

### Log Sanitization

- Never log authentication tokens
- Never log passwords or secrets
- Mask sensitive fields (partial card numbers, etc.)
- Be cautious with user-provided data

### Log Access

- Restrict access to log files
- Use secure log aggregation in production
- Implement log retention policies

## Future Enhancements

- [ ] Log aggregation integration (ELK Stack, Splunk)
- [ ] Real-time log streaming
- [ ] Custom log formats for specific use cases
- [ ] Performance metrics logging
- [ ] Distributed tracing integration
- [ ] Log-based alerting

## Related Files

- `server/utils/logger.js` - Main logger implementation
- `server/index.js` - Express app with logger integration
- `server/middleware/error-handler.js` - Error handling with logging
- `server/tests/utils/logger.test.js` - Test suite

## References

- [Winston Documentation](https://github.com/winstonjs/winston)
- [Winston Daily Rotate File](https://github.com/winstonjs/winston-daily-rotate-file)
- [UUID Library](https://github.com/uuidjs/uuid)
