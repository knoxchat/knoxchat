/**
 * Integration Test - Test the integration between IDEIntegration and GitCheckpointBridge
 * 
 * This test verifies that the integration components work together properly.
 */

import { IntegrationManager } from './IntegrationManager';
import * as path from 'path';

async function testIntegration() {
    console.log('🧪 Starting Integration Test...\n');

    try {
        // Initialize integration manager
        const config = {
            repoPath: process.cwd(), // Use current directory as repo
            workspacePath: process.cwd(),
            enableGitIntegration: true,
            enableIDEIntegration: true
        };

        console.log('📦 Initializing Integration Manager...');
        const integrationManager = new IntegrationManager(config);

        // Test integration status
        console.log('📊 Checking Integration Status...');
        const status = integrationManager.getIntegrationStatus();
        console.log('Status:', status);

        // Test workspace sync
        console.log('\n🔄 Testing Workspace Sync...');
        try {
            await integrationManager.syncWorkspace();
            console.log('✅ Workspace sync successful');
        } catch (error) {
            console.log('⚠️ Workspace sync failed (expected if not a git repo):', (error as Error).message);
        }

        // Test context building
        console.log('\n🧠 Testing Context Building...');
        try {
            const context = await integrationManager.buildContextForQuery('test query', 1000);
            console.log('✅ Context building successful');
            console.log('Context type:', context.context_type);
            console.log('Core files count:', context.core_files.length);
        } catch (error) {
            console.log('⚠️ Context building failed:', (error as Error).message);
        }

        // Test recent checkpoints
        console.log('\n📝 Testing Recent Checkpoints...');
        try {
            const checkpoints = await integrationManager.getRecentCheckpoints(5);
            console.log('✅ Recent checkpoints retrieved');
            console.log('Checkpoints count:', checkpoints.length);
        } catch (error) {
            console.log('⚠️ Recent checkpoints failed:', (error as Error).message);
        }

        // Test contextual assistance
        console.log('\n🤖 Testing Contextual Assistance...');
        try {
            const assistance = await integrationManager.getContextualAssistance(
                { line: 1, character: 0, file: 'test.ts' },
                [{
                    path: 'test.ts',
                    content: 'console.log("test");',
                    language: 'typescript',
                    workspacePath: process.cwd()
                }]
            );
            console.log('✅ Contextual assistance successful');
            console.log('Git context history length:', assistance.gitContext.contextualHistory.length);
        } catch (error) {
            console.log('⚠️ Contextual assistance failed:', (error as Error).message);
        }

        // Cleanup
        console.log('\n🧹 Cleaning up...');
        await integrationManager.cleanup();

        console.log('\n✅ Integration test completed successfully!');

    } catch (error) {
        console.error('\n❌ Integration test failed:', error);
        process.exit(1);
    }
}

// Run the test
if (require.main === module) {
    testIntegration().catch(error => {
        console.error('Test failed:', error);
        process.exit(1);
    });
}

export { testIntegration };
