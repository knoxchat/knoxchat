#!/usr/bin/env node

const readline = require('readline');
const { createSimpleCheckpoint, getConfig } = require('./index.node');

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

console.log('🧪 Interactive Checkpoint System Tester');
console.log('======================================\n');

// Show initial config
console.log('📋 Current Configuration:');
console.log(JSON.stringify(getConfig(), null, 2));
console.log('');

let checkpointCount = 0;
const checkpoints = [];

function showMenu() {
    console.log('\n🎛️  Available Commands:');
    console.log('1. create [description] - Create a new checkpoint');
    console.log('2. list                 - List all created checkpoints');
    console.log('3. config               - Show configuration');
    console.log('4. stats                - Show statistics');
    console.log('5. help                 - Show this menu');
    console.log('6. exit                 - Exit the tester');
    console.log('');
}

function handleCommand(input) {
    const [command, ...args] = input.trim().split(' ');
    
    switch (command.toLowerCase()) {
        case '1':
        case 'create':
            const description = args.join(' ') || `Test checkpoint ${checkpointCount + 1}`;
            try {
                const checkpointId = createSimpleCheckpoint(description);
                checkpointCount++;
                checkpoints.push({
                    id: checkpointId,
                    description,
                    created: new Date().toISOString()
                });
                console.log(`✅ Created checkpoint: ${checkpointId}`);
                console.log(`📝 Description: ${description}`);
            } catch (error) {
                console.error('❌ Error creating checkpoint:', error.message);
            }
            break;
            
        case '2':
        case 'list':
            if (checkpoints.length === 0) {
                console.log('📭 No checkpoints created yet');
            } else {
                console.log(`📋 Created Checkpoints (${checkpoints.length}):`);
                checkpoints.forEach((cp, index) => {
                    console.log(`${index + 1}. ${cp.id.substring(0, 8)}... - ${cp.description}`);
                    console.log(`   Created: ${new Date(cp.created).toLocaleString()}`);
                });
            }
            break;
            
        case '3':
        case 'config':
            console.log('📋 Configuration:');
            console.log(JSON.stringify(getConfig(), null, 2));
            break;
            
        case '4':
        case 'stats':
            console.log('📊 Statistics:');
            console.log(`Total checkpoints created: ${checkpointCount}`);
            console.log(`Checkpoints in memory: ${checkpoints.length}`);
            if (checkpoints.length > 0) {
                console.log(`First checkpoint: ${new Date(checkpoints[0].created).toLocaleString()}`);
                console.log(`Last checkpoint: ${new Date(checkpoints[checkpoints.length - 1].created).toLocaleString()}`);
            }
            break;
            
        case '5':
        case 'help':
            showMenu();
            break;
            
        case '6':
        case 'exit':
        case 'quit':
            console.log('👋 Goodbye!');
            rl.close();
            return;
            
        default:
            console.log('❓ Unknown command. Type "help" for available commands.');
    }
    
    prompt();
}

function prompt() {
    rl.question('checkpoint> ', handleCommand);
}

// Start the interactive session
showMenu();
prompt();
