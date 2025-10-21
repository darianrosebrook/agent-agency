#!/usr/bin/env node

/**
 * @fileoverview Workflow Modules Index
 * Central export point for all workflow generation modules
 * @author @darianrosebrook
 */

// Base workflow components
const {
  createBaseWorkflow,
  getWorkflowTriggers,
  createSetupJob,
} = require('./workflow-base');

// Quality assurance jobs
const {
  createLintJob,
  createTestJob,
  createSecurityJob,
  getTestCommand,
} = require('./quality-jobs');

// Build and deployment jobs
const {
  createBuildJob,
  createDeployJob,
  createDockerJob,
  getBuildCommand,
  getBuildArtifactsPath,
} = require('./build-jobs');

module.exports = {
  // Base workflow
  createBaseWorkflow,
  getWorkflowTriggers,
  createSetupJob,

  // Quality jobs
  createLintJob,
  createTestJob,
  createSecurityJob,
  getTestCommand,

  // Build jobs
  createBuildJob,
  createDeployJob,
  createDockerJob,
  getBuildCommand,
  getBuildArtifactsPath,
};
