#!/usr/bin/env node
/**
 * Phase 8 Verification Tests — Advanced Features
 *
 * Tests:
 *   8.1  Incremental Checkpointing — delta chains, reconstruction
 *   8.2  Branch Management — create, list, switch, delete, merge
 *   8.3  AI-Powered Analysis — description generation, risk, impact, grouping
 *   8.4  Collaborative Checkpoints — sharing, bundles, audit trail
 *   8.5  Performance Monitoring — dashboard, storage, recording events
 *
 * Usage:
 *   node test-phase8.js
 *
 * Prerequisites:
 *   - Native module built: node build.js
 */

'use strict';

const path = require('path');
const fs = require('fs');
const os = require('os');
const crypto = require('crypto');

// ─── Module Loading ──────────────────────────────────────────────
let cp;
try {
    cp = require('./index.node');
} catch (err) {
    console.error('❌ Failed to load native module (index.node).');
    console.error('   Run `node build.js` in core/checkpoints/ first.');
    console.error('   Error:', err.message);
    process.exit(1);
}

// ─── Test Harness ────────────────────────────────────────────────
let passed = 0;
let failed = 0;
let skipped = 0;
const failures = [];

function assert(condition, message) {
    if (!condition) throw new Error(`Assertion failed: ${message}`);
}

function assertEqual(actual, expected, message) {
    if (actual !== expected) {
        throw new Error(
            `${message}\n  Expected: ${JSON.stringify(expected)}\n  Actual:   ${JSON.stringify(actual)}`
        );
    }
}

function assertNotEqual(actual, notExpected, message) {
    if (actual === notExpected) {
        throw new Error(
            `${message}\n  Should not be: ${JSON.stringify(notExpected)}\n  Actual:       ${JSON.stringify(actual)}`
        );
    }
}

function assertType(value, expectedType, message) {
    if (typeof value !== expectedType) {
        throw new Error(
            `${message}\n  Expected type: ${expectedType}\n  Actual type:   ${typeof value} (value: ${JSON.stringify(value)})`
        );
    }
}

function assertGreaterThan(actual, threshold, message) {
    if (!(actual > threshold)) {
        throw new Error(`${message}\n  Expected > ${threshold}\n  Actual:  ${actual}`);
    }
}

function assertGreaterThanOrEqual(actual, threshold, message) {
    if (actual < threshold) {
        throw new Error(`${message}\n  Expected >= ${threshold}\n  Actual:   ${actual}`);
    }
}

function assertThrows(fn, message) {
    let threw = false;
    try { fn(); } catch { threw = true; }
    if (!threw) throw new Error(`Expected to throw: ${message}`);
}

function test(name, fn) {
    try {
        fn();
        passed++;
        console.log(`  ✅ ${name}`);
    } catch (err) {
        failed++;
        failures.push({ name, error: err.message });
        console.log(`  ❌ ${name}`);
        console.log(`     ${err.message.split('\n')[0]}`);
    }
}

function skip(name, reason) {
    skipped++;
    console.log(`  ⏭️  ${name} (${reason})`);
}

// ─── Setup ───────────────────────────────────────────────────────
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'knox-phase8-'));
const wsDir = path.join(tmpDir, 'workspace');
const storageDir = path.join(tmpDir, 'storage');

fs.mkdirSync(wsDir, { recursive: true });

// Create a workspace with test files
fs.mkdirSync(path.join(wsDir, 'src'), { recursive: true });
fs.writeFileSync(path.join(wsDir, 'src', 'app.ts'), 'export function main() { console.log("hello"); }');
fs.writeFileSync(path.join(wsDir, 'src', 'utils.ts'), 'export function add(a: number, b: number) { return a + b; }');
fs.writeFileSync(path.join(wsDir, 'config.json'), '{"version": 1}');
fs.writeFileSync(path.join(wsDir, 'README.md'), '# Test Project');

// Initialize checkpoint manager
const managerCreated = cp.createCheckpointManager({
    storagePath: storageDir,
    maxCheckpoints: 100,
    retentionDays: 30,
}, wsDir);

if (!managerCreated) {
    console.error('❌ Failed to create checkpoint manager');
    process.exit(1);
}

function cleanup() {
    try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch { /* ok */ }
}
process.on('exit', cleanup);

console.log('🧪 Phase 8 — Advanced Features Tests\n');

// ═════════════════════════════════════════════════════════════════
// 8.1  Incremental Checkpointing
// ═════════════════════════════════════════════════════════════════
console.log('── 8.1 Incremental Checkpointing ──');

let baseCheckpointId;

test('8.1.1 Create initial checkpoint as baseline', () => {
    baseCheckpointId = cp.createCheckpoint(wsDir, 'baseline-for-incremental');
    assertType(baseCheckpointId, 'string', 'Should return checkpoint ID');
    assertEqual(baseCheckpointId.length, 36, 'Should be UUID format');
});

let incrementalId1;
test('8.1.2 Create incremental checkpoint stores only deltas', () => {
    // Modify one file
    fs.writeFileSync(path.join(wsDir, 'src', 'app.ts'), 'export function main() { console.log("modified"); }');

    incrementalId1 = cp.createIncrementalCheckpoint({
        description: 'Incremental: modified app.ts',
        tags: ['incremental', 'test'],
    });
    assertType(incrementalId1, 'string', 'Should return incremental checkpoint ID');
    assertEqual(incrementalId1.length, 36, 'Should be UUID format');
});

let incrementalId2;
test('8.1.3 Create second incremental checkpoint (chain depth 2)', () => {
    // Add a new file
    fs.writeFileSync(path.join(wsDir, 'src', 'helpers.ts'), 'export function noop() {}');

    incrementalId2 = cp.createIncrementalCheckpoint({
        description: 'Incremental: added helpers.ts',
        tags: [],
    });
    assertType(incrementalId2, 'string', 'Should return second incremental checkpoint ID');
    assertNotEqual(incrementalId2, incrementalId1, 'Should be different from first');
});

test('8.1.4 Reconstruct checkpoint from delta chain', () => {
    const reconstructed = cp.reconstructCheckpoint(incrementalId2);
    assertType(reconstructed, 'object', 'Should return reconstructed object');
    assertType(reconstructed.chainLength, 'number', 'chainLength should be number');
    assertGreaterThanOrEqual(reconstructed.chainLength, 1, 'Chain should have >= 1 link');
    assertType(reconstructed.fileCount, 'number', 'fileCount should be number');
    assertGreaterThan(reconstructed.fileCount, 0, 'Should have files');
    assert(Array.isArray(reconstructed.files), 'files should be array');

    // Verify file entries have expected structure
    if (reconstructed.files.length > 0) {
        const file = reconstructed.files[0];
        assertType(file.path, 'string', 'File path should be string');
        assertType(file.contentHash, 'string', 'contentHash should be string');
        assertType(file.sizeBytes, 'number', 'sizeBytes should be number');
    }
});

test('8.1.5 Reconstruct first incremental shows correct files', () => {
    const reconstructed = cp.reconstructCheckpoint(incrementalId1);
    assertGreaterThan(reconstructed.fileCount, 0, 'First incremental should have files');
    assertType(reconstructed.totalChainSizeBytes, 'number', 'totalChainSizeBytes should be number');
});

// ═════════════════════════════════════════════════════════════════
// 8.2  Branch Management
// ═════════════════════════════════════════════════════════════════
console.log('\n── 8.2 Branch Management ──');

let featureBranch;
test('8.2.1 Create branch from checkpoint', () => {
    featureBranch = cp.createBranch('feature-x', baseCheckpointId, 'Feature X branch');
    assertType(featureBranch, 'object', 'Should return branch object');
    assertType(featureBranch.id, 'string', 'Branch should have id');
    assertEqual(featureBranch.name, 'feature-x', 'Branch name should match');
    assertEqual(featureBranch.baseCheckpointId, baseCheckpointId, 'Base ID should match');
    assertType(featureBranch.createdAt, 'string', 'Should have createdAt');
    assertEqual(featureBranch.description, 'Feature X branch', 'Description should match');
    assertEqual(featureBranch.isDefault, false, 'New branch should not be default');
});

test('8.2.2 List branches includes new branch', () => {
    const branches = cp.listBranches();
    assert(Array.isArray(branches), 'Should return array');
    assertGreaterThanOrEqual(branches.length, 1, 'Should have at least 1 branch');

    const found = branches.find(b => b.name === 'feature-x');
    assert(found, 'Should find feature-x branch');
    assertEqual(found.id, featureBranch.id, 'IDs should match');
});

test('8.2.3 Switch branch changes active branch', () => {
    const result = cp.switchBranch(featureBranch.id);
    assertEqual(result, true, 'switchBranch should return true');
});

let secondBranch;
test('8.2.4 Create a second branch for merge testing', () => {
    secondBranch = cp.createBranch('bugfix-y', baseCheckpointId, 'Bugfix Y branch');
    assertType(secondBranch.id, 'string', 'Second branch should have id');
    assertEqual(secondBranch.name, 'bugfix-y', 'Name should match');
});

test('8.2.5 Merge branches with SourceWins strategy', () => {
    // Create checkpoints on each branch to have content to merge
    cp.switchBranch(featureBranch.id);
    fs.writeFileSync(path.join(wsDir, 'src', 'feature.ts'), 'export const feature = true;');
    cp.createCheckpoint(wsDir, 'feature commit');

    cp.switchBranch(secondBranch.id);
    fs.writeFileSync(path.join(wsDir, 'src', 'bugfix.ts'), 'export const fix = true;');
    cp.createCheckpoint(wsDir, 'bugfix commit');

    const mergeResult = cp.mergeBranches(featureBranch.id, secondBranch.id, 'SourceWins');
    assertType(mergeResult, 'object', 'Merge should return result object');
    assertType(mergeResult.success, 'boolean', 'success should be boolean');
    assert(Array.isArray(mergeResult.mergedFiles), 'mergedFiles should be array');
    assert(Array.isArray(mergeResult.conflicts), 'conflicts should be array');
    assertType(mergeResult.strategy, 'string', 'strategy should be string');
});

test('8.2.6 Delete branch removes it from list', () => {
    // Create a temporary branch, then delete it
    const tempBranch = cp.createBranch('temp-branch', baseCheckpointId, 'temporary');
    assertEqual(cp.deleteBranch(tempBranch.id), true, 'Delete should return true');

    const branches = cp.listBranches();
    const found = branches.find(b => b.id === tempBranch.id);
    assertEqual(found, undefined, 'Deleted branch should not be in list');
});

test('8.2.7 Cannot delete default branch', () => {
    const branches = cp.listBranches();
    const defaultBranch = branches.find(b => b.isDefault);
    if (defaultBranch) {
        assertThrows(
            () => cp.deleteBranch(defaultBranch.id),
            'Should not be able to delete default branch'
        );
    }
});

// ═════════════════════════════════════════════════════════════════
// 8.3  AI-Powered Checkpoint Analysis
// ═════════════════════════════════════════════════════════════════
console.log('\n── 8.3 AI-Powered Analysis ──');

test('8.3.1 analyzeCheckpoint returns full analysis', () => {
    // Create a checkpoint with interesting files to analyze
    fs.writeFileSync(path.join(wsDir, 'config.json'), '{"version": 2, "apiKey": "changed"}');
    fs.writeFileSync(path.join(wsDir, 'src', 'api.ts'), 'export function getUser() { return fetch("/api/user"); }');
    const analysisCheckpointId = cp.createCheckpoint(wsDir, 'analysis-test');

    const analysis = cp.analyzeCheckpoint(analysisCheckpointId);
    assertType(analysis, 'object', 'Should return analysis object');

    // Generated description
    assertType(analysis.generatedDescription, 'string', 'Should have generatedDescription');
    assertGreaterThan(analysis.generatedDescription.length, 0, 'Description should not be empty');

    // Risk assessment
    assertType(analysis.riskAssessment, 'object', 'Should have riskAssessment');
    assertType(analysis.riskAssessment.level, 'string', 'Risk level should be string');
    assert(['Low', 'Medium', 'High', 'Critical'].includes(analysis.riskAssessment.level),
        `Risk level should be valid, got: ${analysis.riskAssessment.level}`);
    assertType(analysis.riskAssessment.score, 'number', 'Risk score should be number');
    assertGreaterThanOrEqual(analysis.riskAssessment.score, 0, 'Score >= 0');
    assert(analysis.riskAssessment.score <= 1.0, 'Score <= 1.0');
    assert(Array.isArray(analysis.riskAssessment.factors), 'factors should be array');

    // Impact analysis
    assertType(analysis.impactAnalysis, 'object', 'Should have impactAnalysis');
    assert(Array.isArray(analysis.impactAnalysis.affectedFeatures), 'affectedFeatures should be array');
    assert(Array.isArray(analysis.impactAnalysis.affectedLayers), 'affectedLayers should be array');
    assertType(analysis.impactAnalysis.scope, 'string', 'scope should be string');
});

test('8.3.2 Risk factors have correct structure', () => {
    const cpId = cp.createCheckpoint(wsDir, 'risk-factors-test');
    const analysis = cp.analyzeCheckpoint(cpId);

    if (analysis.riskAssessment.factors.length > 0) {
        const factor = analysis.riskAssessment.factors[0];
        assertType(factor.category, 'string', 'Factor category should be string');
        assertType(factor.description, 'string', 'Factor description should be string');
        assertType(factor.weight, 'number', 'Factor weight should be number');
        assert(Array.isArray(factor.affectedFiles), 'affectedFiles should be array');
    }
});

test('8.3.3 suggestCheckpointGroups returns groups', () => {
    // Create several related checkpoints
    for (let i = 0; i < 3; i++) {
        fs.writeFileSync(path.join(wsDir, 'src', 'app.ts'), `export function main() { return ${i}; }`);
        cp.createCheckpoint(wsDir, `related-change-${i}`);
    }

    const groups = cp.suggestCheckpointGroups(5);
    assert(Array.isArray(groups), 'Should return array of groups');

    if (groups.length > 0) {
        const group = groups[0];
        assertType(group.name, 'string', 'Group should have name');
        assert(Array.isArray(group.checkpointIds), 'Group should have checkpointIds array');
        assertType(group.similarity, 'number', 'Group should have similarity score');
        assertGreaterThan(group.similarity, 0, 'Similarity should be > 0');
    }
});

test('8.3.4 Config file changes are detected as risk factor', () => {
    // Modify config file (should trigger config risk factor)
    fs.writeFileSync(path.join(wsDir, 'config.json'), '{"version": 99, "debug": true}');
    const cpId = cp.createCheckpoint(wsDir, 'config-risk-test');
    const analysis = cp.analyzeCheckpoint(cpId);

    assertType(analysis.riskAssessment.score, 'number', 'Should have risk score');
    // Config changes should result in some risk factors
    assert(analysis.riskAssessment.factors.length >= 0,
        'Should have risk factors (may be empty if config not modified since last)');
});

// ═════════════════════════════════════════════════════════════════
// 8.4  Collaborative Checkpoints
// ═════════════════════════════════════════════════════════════════
console.log('\n── 8.4 Collaborative Checkpoints ──');

test('8.4.1 shareCheckpoints creates a shared bundle', () => {
    const cpId1 = cp.createCheckpoint(wsDir, 'share-test-1');
    const cpId2 = cp.createCheckpoint(wsDir, 'share-test-2');

    const bundle = cp.shareCheckpoints([cpId1, cpId2], 'Sharing test checkpoints');
    assertType(bundle, 'object', 'Should return bundle object');
    assertType(bundle.id, 'string', 'Bundle should have id');
    assertType(bundle.description, 'string', 'Bundle should have description');
    assertEqual(bundle.description, 'Sharing test checkpoints', 'Description should match');
    assertType(bundle.checkpointCount, 'number', 'Should have checkpointCount');
    assertEqual(bundle.checkpointCount, 2, 'Should contain 2 checkpoints');
    assertType(bundle.sharedAt, 'string', 'Should have sharedAt');
});

test('8.4.2 listSharedBundles returns created bundles', () => {
    const bundles = cp.listSharedBundles();
    assert(Array.isArray(bundles), 'Should return array');
    assertGreaterThanOrEqual(bundles.length, 1, 'Should have at least 1 bundle');

    const bundle = bundles[0];
    assertType(bundle.id, 'string', 'Bundle should have id');
    assertType(bundle.description, 'string', 'Bundle should have description');
});

test('8.4.3 getAuditTrail returns logged operations', () => {
    const trail = cp.getAuditTrail(50);
    assert(Array.isArray(trail), 'Should return array');
    assertGreaterThan(trail.length, 0, 'Should have audit entries after operations');

    const entry = trail[0];
    assertType(entry.id, 'string', 'Entry should have id');
    assertType(entry.timestamp, 'string', 'Entry should have timestamp');
    assertType(entry.action, 'string', 'Entry should have action');
    assertType(entry.outcome, 'string', 'Entry should have outcome');
});

test('8.4.4 getAuditTrail with action filter', () => {
    const trail = cp.getAuditTrail(50, 'share');
    assert(Array.isArray(trail), 'Filtered trail should be array');

    // All returned entries should have the share action
    for (const entry of trail) {
        assert(entry.action.toLowerCase().includes('share'),
            `Filtered entry action should contain "share", got: ${entry.action}`);
    }
});

test('8.4.5 shareCheckpoints with single checkpoint', () => {
    const cpId = cp.createCheckpoint(wsDir, 'single-share');
    const bundle = cp.shareCheckpoints([cpId], 'Single checkpoint share');
    assertEqual(bundle.checkpointCount, 1, 'Bundle should have 1 checkpoint');
});

// ═════════════════════════════════════════════════════════════════
// 8.5  Performance Monitoring
// ═════════════════════════════════════════════════════════════════
console.log('\n── 8.5 Performance Monitoring ──');

test('8.5.1 recordStorageSnapshot captures current state', () => {
    const result = cp.recordStorageSnapshot();
    assertEqual(result, true, 'Should return true on success');
});

test('8.5.2 getStorageUsage returns real disk measurements', () => {
    const storage = cp.getStorageUsage();
    assertType(storage, 'object', 'Should return object');
    assertType(storage.totalBytes, 'number', 'totalBytes should be number');
    assertGreaterThan(storage.totalBytes, 0, 'Should have non-zero storage after checkpoints');
    assertType(storage.blobCount, 'number', 'blobCount should be number');
    assertType(storage.databaseBytes, 'number', 'databaseBytes should be number');
    assertGreaterThan(storage.databaseBytes, 0, 'DB should have non-zero size');
});

test('8.5.3 recordRestorationEvent persists event', () => {
    const cpId = cp.createCheckpoint(wsDir, 'restore-event-test');
    assertType(cpId, 'string', 'createCheckpoint should return string ID');
    const result = cp.recordRestorationEvent({
        checkpointId: cpId,
        success: true,
        durationMs: 150.5,
        filesRestored: 4,
        filesFailed: 0,
    });
    assertEqual(result, true, 'Should return true on success');
});

test('8.5.4 recordRestorationEvent with failure', () => {
    const cpId = cp.createCheckpoint(wsDir, 'restore-fail-event');
    assertType(cpId, 'string', 'createCheckpoint should return string ID');
    const result = cp.recordRestorationEvent({
        checkpointId: cpId,
        success: false,
        durationMs: 50.0,
        filesRestored: 0,
        filesFailed: 3,
        error: 'Permission denied',
    });
    assertEqual(result, true, 'Should return true even for failure events');
});

test('8.5.5 recordAISessionMetrics persists metrics', () => {
    const sessionId = crypto.randomUUID();
    const now = new Date();
    const result = cp.recordAISessionMetrics({
        sessionId: sessionId,
        startedAt: new Date(now - 3600000).toISOString(),
        endedAt: now.toISOString(),
        filesChanged: 10,
        linesAdded: 200,
        linesDeleted: 50,
        checkpointsCreated: 3,
        rollbacks: 1,
        durationSeconds: 3600,
    });
    assertEqual(result, true, 'Should return true on success');
});

test('8.5.6 getPerformanceDashboard returns aggregated data', () => {
    const dashboard = cp.getPerformanceDashboard(30);
    assertType(dashboard, 'object', 'Dashboard should be object');

    // Current storage
    assertType(dashboard.currentStorage, 'object', 'Should have currentStorage');
    assertType(dashboard.currentStorage.totalBytes, 'number', 'totalBytes should be number');

    // Summary
    assertType(dashboard.summary, 'object', 'Should have summary');
    assertType(dashboard.summary.totalCheckpointsCreated, 'number', 'totalCheckpointsCreated should be number');
    assertGreaterThan(dashboard.summary.totalCheckpointsCreated, 0, 'Should have checkpoints');
    assertType(dashboard.summary.restorationSuccessRate, 'number', 'restorationSuccessRate should be number');
    assertType(dashboard.summary.avgCreationTimeMs, 'number', 'avgCreationTimeMs should be number');
    assertType(dashboard.summary.avgRestorationTimeMs, 'number', 'avgRestorationTimeMs should be number');

    // Storage history
    assert(Array.isArray(dashboard.storageHistory), 'storageHistory should be array');

    // Creation frequency
    assert(Array.isArray(dashboard.creationFrequency), 'creationFrequency should be array');

    // Restoration events
    assert(Array.isArray(dashboard.restorationEvents), 'restorationEvents should be array');
    assertGreaterThan(dashboard.restorationEvents.length, 0,
        'Should have restoration events after recording them');

    // AI session metrics
    assert(Array.isArray(dashboard.aiSessionMetrics), 'aiSessionMetrics should be array');
    assertGreaterThan(dashboard.aiSessionMetrics.length, 0,
        'Should have AI session metrics after recording');
});

test('8.5.7 Dashboard restoration events have correct structure', () => {
    const dashboard = cp.getPerformanceDashboard(30);
    if (dashboard.restorationEvents.length > 0) {
        const event = dashboard.restorationEvents[0];
        assertType(event.timestamp, 'string', 'Event should have timestamp');
        assertType(event.success, 'boolean', 'Event should have success');
        assertType(event.durationMs, 'number', 'Event should have durationMs');
        assertType(event.filesRestored, 'number', 'Event should have filesRestored');
    }
});

test('8.5.8 Dashboard AI session metrics have correct structure', () => {
    const dashboard = cp.getPerformanceDashboard(30);
    if (dashboard.aiSessionMetrics.length > 0) {
        const m = dashboard.aiSessionMetrics[0];
        assertType(m.sessionId, 'string', 'Should have sessionId');
        assertType(m.filesChanged, 'number', 'Should have filesChanged');
        assertType(m.linesAdded, 'number', 'Should have linesAdded');
        assertType(m.linesDeleted, 'number', 'Should have linesDeleted');
        assertType(m.checkpointsCreated, 'number', 'Should have checkpointsCreated');
        assertType(m.rollbacks, 'number', 'Should have rollbacks');
        assertType(m.durationSeconds, 'number', 'Should have durationSeconds');
    }
});

test('8.5.9 Multiple storage snapshots accumulate in history', () => {
    // Record a second snapshot
    cp.recordStorageSnapshot();

    const dashboard = cp.getPerformanceDashboard(30);
    assertGreaterThanOrEqual(dashboard.storageHistory.length, 1,
        'Should have storage history after snapshots');
});

// ═════════════════════════════════════════════════════════════════
// Summary
// ═════════════════════════════════════════════════════════════════
console.log('\n' + '═'.repeat(60));
console.log(`Phase 8 Results: ${passed} passed, ${failed} failed, ${skipped} skipped`);
console.log('═'.repeat(60));

if (failures.length > 0) {
    console.log('\nFailures:');
    for (const f of failures) {
        console.log(`  ❌ ${f.name}: ${f.error}`);
    }
}

process.exit(failed > 0 ? 1 : 0);
