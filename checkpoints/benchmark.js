#!/usr/bin/env node

const { createSimpleCheckpoint, getConfig } = require('./index.node');

console.log('⚡ Checkpoint System Performance Benchmark');
console.log('==========================================\n');

// Configuration test
console.log('📋 Testing configuration retrieval...');
console.time('Config retrieval');
const config = getConfig();
console.timeEnd('Config retrieval');
console.log('Config:', config);
console.log('');

// Single checkpoint creation test
console.log('📸 Testing single checkpoint creation...');
console.time('Single checkpoint');
const singleCheckpoint = createSimpleCheckpoint('Performance test');
console.timeEnd('Single checkpoint');
console.log('Checkpoint ID:', singleCheckpoint);
console.log('');

// Batch checkpoint creation test
console.log('📦 Testing batch checkpoint creation...');
const batchSizes = [10, 100, 1000];

for (const batchSize of batchSizes) {
    console.log(`\n🔄 Creating ${batchSize} checkpoints...`);
    const startTime = Date.now();
    
    const checkpoints = [];
    for (let i = 0; i < batchSize; i++) {
        const id = createSimpleCheckpoint(`Batch test ${i + 1}/${batchSize}`);
        checkpoints.push(id);
    }
    
    const endTime = Date.now();
    const totalTime = endTime - startTime;
    console.log(`⏱️  Total time: ${totalTime}ms`);
    console.log(`✅ Created ${checkpoints.length} checkpoints`);
    const avgTime = totalTime / batchSize;
    console.log(`📊 Average: ${avgTime.toFixed(2)}ms per checkpoint`);
}

// Memory usage test
console.log('\n💾 Memory usage:');
const memUsage = process.memoryUsage();
console.log(`RSS: ${(memUsage.rss / 1024 / 1024).toFixed(2)} MB`);
console.log(`Heap Used: ${(memUsage.heapUsed / 1024 / 1024).toFixed(2)} MB`);
console.log(`Heap Total: ${(memUsage.heapTotal / 1024 / 1024).toFixed(2)} MB`);
console.log(`External: ${(memUsage.external / 1024 / 1024).toFixed(2)} MB`);

// Error handling test
console.log('\n🛡️  Testing error handling...');
try {
    // Test with empty description
    createSimpleCheckpoint('');
    console.log('✅ Empty description handled correctly');
} catch (error) {
    console.log('⚠️  Empty description error:', error.message);
}

try {
    // Test with very long description
    const longDesc = 'A'.repeat(10000);
    const longId = createSimpleCheckpoint(longDesc);
    console.log('✅ Long description handled correctly:', longId.substring(0, 8) + '...');
} catch (error) {
    console.log('⚠️  Long description error:', error.message);
}

console.log('\n🎉 Benchmark completed!');
