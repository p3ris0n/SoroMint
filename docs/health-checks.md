# Health Check and Network Metadata API

## Overview
The health check system provides real-time monitoring and metadata reporting for the SoroMint backend ecosystem. It is designed to be utilized by infrastructure monitors, frontend dashboards, and automated deployment pipelines.

## Endpoint: `/api/health`

### Specification
- **Method**: GET
- **Access**: Public
- **Authentication**: None Required

### Core Components Monitored
*   **Database**: Connectivity status to MongoDB.
*   **Stellar Network**: Reports current active Stellar network configuration.
*   **System Metrics**: Includes server version, uptime, and current server time.

### Response Format (JSON)

#### Example Healthy Response (200 OK)
```json
{
  "status": "healthy",
  "timestamp": "2026-03-24T10:30:56.000Z",
  "version": "1.0.0",
  "uptime": "0h 15m 30s",
  "services": {
    "database": {
      "status": "up",
      "connection": "connected"
    },
    "stellar": {
      "network": "Test SDF Network ; September 2015"
    }
  }
}
```

#### Example Unhealthy Response (503 Service Unavailable)
*Sent when MongoDB is disconnected.*
```json
{
  "status": "unhealthy",
  "timestamp": "2026-03-24T10:45:12.000Z",
  "version": "1.0.0",
  "uptime": "1h 5m 22s",
  "services": {
    "database": {
      "status": "down",
      "connection": "disconnected"
    },
    "stellar": {
      "network": "Test SDF Network ; September 2015"
    }
  }
}
```

## Monitoring Configuration
For standard monitoring tools, integrate by checking for the `200` HTTP status code on the `/api/health` endpoint. A return of `503` indicates critical service failure.

## Implementation Details
Integrated via `server/routes/status-routes.js` using `mongoose.connection.readyState` for database state monitoring and `process.uptime()` for system metrics.
