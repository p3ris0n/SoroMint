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

const app = express();
const PORT = process.env.PORT || 5000;

// Middleware
app.use(cors());
app.use(express.json());

// Logging middleware (must be early in the chain)
app.use(correlationIdMiddleware);
app.use(httpLoggerMiddleware);

// Database Connection
mongoose.connect(process.env.MONGO_URI || 'mongodb://localhost:27017/soromint')
  .then(() => {
    logDatabaseConnection(true);
  })
  .catch(err => {
    logDatabaseConnection(false, err);
  });

// Routes
app.get('/api/tokens/:owner', asyncHandler(async (req, res) => {
  logger.info('Fetching tokens for owner', {
    correlationId: req.correlationId,
    ownerPublicKey: req.params.owner
  });
  const tokens = await Token.find({ ownerPublicKey: req.params.owner });
  res.json(tokens);
}));

app.post('/api/tokens', asyncHandler(async (req, res) => {
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
  res.json(newToken);
}));

app.get('/api/status', (req, res) => {
  logger.debug('Status check requested', { correlationId: req.correlationId });
  res.json({ status: 'Server is running', network: process.env.NETWORK_PASSPHRASE });
});

// 404 handler for undefined routes
app.use(notFoundHandler);

// Centralized error handling middleware (must be last)
app.use(errorHandler);

app.listen(PORT, () => {
  logStartupInfo(PORT, process.env.NETWORK_PASSPHRASE || 'Unknown Network');
});
