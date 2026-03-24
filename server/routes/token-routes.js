const express = require("express");
const Token = require("../models/Token");
const DeploymentAudit = require("../models/DeploymentAudit");
const { asyncHandler, AppError } = require("../middleware/error-handler");
const { logger } = require("../utils/logger");
const { authenticate } = require("../middleware/auth");
const {
  validateToken,
  validatePagination,
} = require("../validators/token-validator");

const router = express.Router();

/**
 * @route GET /api/tokens/:owner
 * @group Tokens - Token management operations
 * @param {string} owner.path - Owner's Stellar public key
 * @param {number} page.query - Page number (default: 1)
 * @param {number} limit.query - Items per page (default: 20)
 * @returns {Object} 200 - Paginated tokens with metadata
 * @returns {Error} 400 - Invalid parameters
 * @returns {Error} default - Unexpected error
 * @security [JWT]
 */
router.get(
  "/tokens/:owner",
  authenticate,
  validatePagination,
  asyncHandler(async (req, res) => {
    const { owner } = req.params;
    const { page, limit } = req.query;

    logger.info("Fetching tokens for owner", {
      correlationId: req.correlationId,
      ownerPublicKey: owner,
      page,
      limit,
    });

    const skip = (page - 1) * limit;

    const [tokens, totalCount] = await Promise.all([
      Token.find({ ownerPublicKey: owner })
        .sort({ createdAt: -1 })
        .skip(skip)
        .limit(limit),
      Token.countDocuments({ ownerPublicKey: owner }),
    ]);

    const totalPages = Math.ceil(totalCount / limit);

    res.json({
      success: true,
      data: tokens,
      metadata: {
        totalCount,
        page,
        totalPages,
        limit,
      },
    });
  }),
);

/**
 * @route POST /api/tokens
 * @group Tokens - Token management operations
 * @param {TokenCreateInput.model} body.required - Token creation data
 * @returns {Token} 201 - Successfully created token
 * @returns {Error} 400 - Missing required fields or validation error
 * @returns {Error} 409 - Token with this contractId already exists
 * @returns {Error} default - Unexpected error
 * @security [JWT]
 */
router.post(
  "/tokens",
  authenticate,
  validateToken,
  asyncHandler(async (req, res) => {
    const { name, symbol, decimals, contractId, ownerPublicKey } = req.body;
    const userId = req.user._id;

    logger.info("Creating new token", {
      correlationId: req.correlationId,
      name,
      symbol,
      ownerPublicKey,
      userId,
    });

    try {
      const newToken = new Token({
        name,
        symbol,
        decimals,
        contractId,
        ownerPublicKey,
      });
      await newToken.save();

      logger.info("Token created successfully", {
        correlationId: req.correlationId,
        tokenId: newToken._id,
      });

      // Log successful deployment
      await DeploymentAudit.create({
        userId,
        tokenName: name,
        contractId,
        status: "SUCCESS",
      });

      res.status(201).json(newToken);
    } catch (error) {
      logger.error("Token creation failed", {
        correlationId: req.correlationId,
        error: error.message,
      });

      // Log failed deployment attempt
      await DeploymentAudit.create({
        userId,
        tokenName: name,
        contractId,
        status: "FAIL",
        errorMessage: error.message,
      });

      // Re-throw to be handled by error middleware
      throw error;
    }
  }),
);

module.exports = router;
