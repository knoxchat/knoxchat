const { createSimpleCheckpoint, getConfig } = require('./index.node');

console.log('🧪 Testing Knox Checkpoint System...\n');

try {
    // Test configuration
    console.log('📋 Getting configuration...');
    const config = getConfig();
    console.log('✅ Config:', JSON.stringify(config, null, 2));
    
    // Test checkpoint creation
    console.log('\n📸 Creating checkpoint...');
    const checkpointId = createSimpleCheckpoint('Test checkpoint');
    console.log('✅ Created checkpoint:', checkpointId);
    
    console.log('\n🎉 All tests passed! Checkpoint system is working.');
    
} catch (error) {
    console.error('❌ Test failed:', error);
    process.exit(1);
}
