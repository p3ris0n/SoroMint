const express = require('express');
const mongoose = require('mongoose');
const cors = require('cors');
require('dotenv').config();

const Token = require('./models/Token');
const stellarService = require('./services/stellar-service');
const { errorHandler, notFoundHandler, asyncHandler, AppError } = require('./middleware/error-handler');
const {
  logger,
  correlationIdMiddleware,
  httpLoggerMiddleware,
  logStartupInfo,
  logDatabaseConnection
} = require('./utils/logger');
const { setupSwagger } = require('./config/swagger');
const { authenticate } = require('./middleware/auth');
const authRoutes = require('./routes/auth-routes');
const statusRoutes = require('./routes/status-routes');

const app = express();
const PORT = process.env.PORT || 5000;

// Middleware
app.use(cors());
app.use(express.json());

// Logging middleware (must be early in the chain)
app.use(correlationIdMiddleware);
app.use(httpLoggerMiddleware);

// Set up API Documentation
setupSwagger(app);

// Database Connection
mongoose.connect(process.env.MONGO_URI || 'mongodb://localhost:27017/soromint')
  .then(() => {
    logDatabaseConnection(true);
  })
  .catch(err => {
    logDatabaseConnection(false, err);
  });

// Routes
app.use('/api', statusRoutes);
app.use('/api/auth', authRoutes);

/**
 * @route GET /api/tokens/:owner
 * @group Tokens - Token management operations
 * @param {string} owner.path - Owner's Stellar public key
 * @returns {Array.<Token>} 200 - Array of tokens owned by the specified address
 * @returns {Error} 400 - Invalid owner public key format
 * @returns {Error} default - Unexpected error
 * @security [JWT]
 * @example
 * // Response example
 * [
 *   {
 *     "_id": "507f1f77bcf86cd799439011",
 *     "name": "SoroMint Token",
 *     "symbol": "SORO",
 *     "decimals": 7,
 *     "contractId": "CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE",
 *     "ownerPublicKey": "GBZ4XGQW5X6V7Y2Z3A4B5C6D7E8F9G0H1I2J3K4L5M6N7O8P9Q0R1S2T",
 *     "createdAt": "2024-01-15T10:30:00.000Z"
 *   }
 * ]
 */
app.get('/api/tokens/:owner', authenticate, asyncHandler(async (req, res) => {
  logger.info('Fetching tokens for owner', {
    correlationId: req.correlationId,
    ownerPublicKey: req.params.owner
  });
  const tokens = await Token.find({ ownerPublicKey: req.params.owner });
  res.json(tokens);
}));

/**
 * @route POST /api/tokens
 * @group Tokens - Token management operations
 * @param {TokenCreateInput.model} body.required - Token creation data
 * @returns {Token} 201 - Successfully created token
 * @returns {Error} 400 - Missing required fields or validation error
 * @returns {Error} 409 - Token with this contractId already exists
 * @returns {Error} default - Unexpected error
 * @security [JWT]
 * @example
 * // Request body
 * {
 *   "name": "SoroMint Token",
 *   "symbol": "SORO",
 *   "decimals": 7,
 *   "contractId": "CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE",
 *   "ownerPublicKey": "GBZ4XGQW5X6V7Y2Z3A4B5C6D7E8F9G0H1I2J3K4L5M6N7O8P9Q0R1S2T"
 * }
 * @example
 * // Response example
 * {
 *   "_id": "507f1f77bcf86cd799439011",
 *   "name": "SoroMint Token",
 *   "symbol": "SORO",
 *   "decimals": 7,
 *   "contractId": "CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE",
 *   "ownerPublicKey": "GBZ4XGQW5X6V7Y2Z3A4B5C6D7E8F9G0H1I2J3K4L5M6N7O8P9Q0R1S2T",
 *   "createdAt": "2024-01-15T10:30:00.000Z"
 * }
 */
app.post('/api/tokens', authenticate, asyncHandler(async (req, res) => {
  const { name, symbol, decimals, contractId, ownerPublicKey } = req.body;

  logger.info('Creating new token', {
    correlationId: req.correlationId,
    name,
    symbol,
    ownerPublicKey
  });

  // Validate required fields
  if (!name || !symbol || !ownerPublicKey) {
    logger.warn('Token creation failed - missing required fields', {
      correlationId: req.correlationId,
      missingFields: { name: !name, symbol: !symbol, ownerPublicKey: !ownerPublicKey }
    });
    throw new AppError('Missing required fields: name, symbol, and ownerPublicKey are required', 400, 'VALIDATION_ERROR');
  }

  const newToken = new Token({ name, symbol, decimals, contractId, ownerPublicKey });
  await newToken.save();
  logger.info('Token created successfully', {
    correlationId: req.correlationId,
    tokenId: newToken._id
  });
  res.status(201).json(newToken);
}));

// 404 handler for undefined routes
app.use(notFoundHandler);

// Centralized error handling middleware (must be last)
app.use(errorHandler);

app.listen(PORT, () => {
  logStartupInfo(PORT, process.env.NETWORK_PASSPHRASE || 'Unknown Network');
  console.log(`🚀 Server running on http://localhost:${PORT}`);
  console.log(`📚 API Documentation available at http://localhost:${PORT}/api-docs`);
});
