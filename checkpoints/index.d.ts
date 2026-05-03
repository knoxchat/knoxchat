/**
 * TypeScript definitions for the Knox Checkpoint System
 */

export interface CheckpointConfig {
  /** Path to store checkpoint data */
  storagePath: string;
  /** Maximum number of checkpoints to keep */
  maxCheckpoints: number;
  /** Maximum age of checkpoints before cleanup (days) */
  retentionDays: number;
  /** Maximum storage size in bytes */
  maxStorageBytes?: number;
  /** Enable compression for checkpoints */
  enableCompression?: boolean;
  /** File extensions to track */
  trackedExtensions?: string[];
  /** Enable debug mode */
  debugMode?: boolean;
}

export interface AgentCheckpointOptions {
  description?: string;
  tags?: string[];
  maxFiles?: number;
}

/**
 * Create a simple checkpoint with the given description
 * Returns the checkpoint ID
 */
export function createSimpleCheckpoint(description: string): string;

/**
 * Get the default checkpoint configuration
 */
export function getConfig(): CheckpointConfig;

/**
 * Initialize the checkpoint manager with configuration
 * Must be called before using other checkpoint functions
 */
export function createCheckpointManager(config: CheckpointConfig, workspacePath: string): boolean;

/**
 * Create an agent checkpoint for AI-generated changes
 */
export function createAgentCheckpoint(options: AgentCheckpointOptions): string;

/**
 * Start an agent session for tracking changes
 */
export function startAgentSession(sessionId: string): boolean;

/**
 * Stop the current agent session
 */
export function stopAgentSession(): boolean;

/**
 * Set the operation mode (Agent, Chat, or Manual)
 */
export function setOperationMode(mode: 'Agent' | 'Chat' | 'Manual'): boolean;

/**
 * Track specific files for AI changes
 */
export function trackAIFiles(filePaths: string[]): boolean;

/**
 * Check if there are pending AI changes
 */
export function hasAIChanges(): boolean;

/**
 * Get changeset statistics
 */
export function getChangesetStats(): { filesTracked: number; changesDetected: number };

/**
 * Run storage garbage collection to remove orphaned content blobs.
 * Returns the number of bytes freed.
 */
export function runStorageGc(): number;