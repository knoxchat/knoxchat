# Knox Checkpoint System - Quick Start Guide

## What is the Checkpoint System?

The Knox Checkpoint System is a **standalone version control** system built specifically for AI-assisted development. It automatically captures snapshots of your code changes, allowing you to safely experiment and easily revert if needed.

### Key Features

✅ **No Git Required** - Works independently without any Git installation  
✅ **AI-Aware** - Designed for tracking AI-generated changes  
✅ **Real-Time Tracking** - Instant file change detection  
✅ **Smart Filtering** - Automatic ignore patterns for your project type  
✅ **Performance Optimized** - Fast scanning with configurable limits  
✅ **User Friendly** - Easy-to-use VSCode commands  

## Quick Start (5 Minutes)

### 1. Create .knoxignore File

Open VSCode Command Palette (`Cmd+Shift+P` or `Ctrl+Shift+P`) and run:

```
Knox Checkpoint: Create .knoxignore
```

The system will:
- Auto-detect your project type (Node.js, Python, Rust, etc.)
- Generate appropriate ignore patterns
- Open the file for you to review

### 2. Configure for Your Project

**Small Project (<1000 files)?** → Default settings work great!  
**Large Project (>1000 files)?** → Run configuration command:

```
Knox Checkpoint: Show Configuration
```

Select options to:
- Adjust scan depth (default: 10 levels)
- Change max file size (default: 1MB)
- View performance metrics

### 3. Start Using Checkpoints

Checkpoints are created automatically when you interact with the AI. You can also create manual checkpoints:

```
Knox Checkpoint: Create Manual Checkpoint
```

### 4. View and Restore

To see all checkpoints and restore one:

```
Knox Checkpoint: Show Checkpoint List
```

Select a checkpoint to restore your code to that point in time.

## Available Commands

All commands are available through the VSCode Command Palette (`Cmd+Shift+P` or `Ctrl+Shift+P`):

### Configuration Commands

| Command | Description |
|---------|-------------|
| `Knox Checkpoint: Show Configuration` | Interactive configuration panel |
| `Knox Checkpoint: Create .knoxignore` | Generate ignore patterns for your project |
| `Knox Checkpoint: Configure Scan Depth` | Set maximum directory depth to scan |
| `Knox Checkpoint: Configure Max File Size` | Set maximum file size to track |
| `Knox Checkpoint: Show Performance Metrics` | View system performance statistics |
| `Knox Checkpoint: Reset Configuration` | Reset all settings to defaults |

### Checkpoint Management

| Command | Description |
|---------|-------------|
| `Knox Checkpoint: Create Manual Checkpoint` | Create a checkpoint manually |
| `Knox Checkpoint: Show Checkpoint List` | View and restore checkpoints |
| `Knox Checkpoint: Cleanup Old Checkpoints` | Remove old checkpoints |
| `Knox Checkpoint: Export Checkpoints` | Export checkpoints to backup file |
| `Knox Checkpoint: Import Checkpoints` | Import checkpoints from backup |

## Configuration Options

### Scan Depth Limit

**What it does**: Limits how deep the system scans into nested directories  
**Default**: 10 levels  
**Range**: 1-50 levels  
**When to adjust**: 
- Increase for projects with very deep folder structures
- Decrease for faster scanning in large monorepos

**Example**:
```typescript
manager.setMaxScanDepth(15); // Scan up to 15 levels deep
```

### Max File Size

**What it does**: Skips files larger than this size  
**Default**: 1 MB  
**Range**: 1 KB - 100 MB  
**When to adjust**:
- Increase if you have large source files
- Decrease for faster scanning

**Presets**:
- 512 KB - Conservative (fastest)
- 1 MB - Default (balanced)
- 2 MB - Moderate
- 5 MB - Large files
- 10 MB - Very large files

**Example**:
```typescript
manager.setMaxFileSize(2 * 1024 * 1024); // 2 MB limit
```

## .knoxignore File

### What is it?

Similar to `.gitignore`, the `.knoxignore` file tells the checkpoint system which files and directories to ignore.

### Auto-Generated Presets

We provide presets for common project types:

| Project Type | Auto-Detects | Key Patterns |
|-------------|--------------|--------------|
| **Node.js / TypeScript** | `package.json` | `node_modules/`, `dist/`, `build/` |
| **Python** | `requirements.txt`, `setup.py` | `__pycache__/`, `venv/`, `.pytest_cache/` |
| **Rust** | `Cargo.toml` | `target/`, `Cargo.lock` |
| **Java** | `pom.xml`, `build.gradle` | `target/`, `.gradle/`, `build/` |
| **Go** | `go.mod` | `bin/`, `pkg/`, `vendor/` |
| **C/C++** | `CMakeLists.txt`, `Makefile` | `build/`, `*.o`, `*.obj` |
| **.NET** | `*.csproj`, `*.sln` | `bin/`, `obj/`, `.vs/` |
| **Ruby** | `Gemfile` | `.bundle/`, `vendor/bundle/` |
| **PHP** | `composer.json` | `vendor/`, `storage/` |

### Example .knoxignore

```gitignore
# Version Control
.git/

# Knox Internal
.knox/
.knox-debug/

# Node.js
node_modules/
dist/
build/

# Environment
.env
.env.local

# Logs
*.log
*.tmp
```

### Pattern Syntax

- `directory/` - Ignore a directory
- `*.ext` - Ignore all files with extension
- `**/pattern` - Match at any depth
- `!file.txt` - Negate (include file)

## Performance Metrics

### What's Tracked

The system tracks:
- **Total Scans** - Number of filesystem scans performed
- **Files Scanned** - Total files examined
- **Last Scan Duration** - Time taken for most recent scan
- **Average Scan Duration** - Average time across all scans
- **Last Scan Timestamp** - When the last scan occurred

### Viewing Metrics

Run command:
```
Knox Checkpoint: Show Performance Metrics
```

Example output:
```
📊 Checkpoint Performance Metrics

Total Scans: 42
Files Scanned: 1,250
Last Scan: 150ms
Average Scan: 125.50ms
Last Scan Time: 11/9/2023, 10:30:00 AM

Efficiency: 29.8 files/scan
```

### Interpreting Metrics

**Good Performance**:
- Scan duration < 200ms
- Efficiency > 20 files/scan
- Consistent scan times

**Needs Optimization**:
- Scan duration > 1000ms
- Efficiency < 10 files/scan
- Increasing scan times

**Optimization Tips**:
1. Reduce scan depth
2. Lower file size limit
3. Add more ignore patterns
4. Check for very deep directory structures

## Real-Time File Tracking

The checkpoint system uses VSCode's FileSystemWatcher for real-time tracking:

### How it Works

1. **File Changed** → Watcher detects change → Adds to tracking list
2. **File Created** → Watcher detects creation → Adds to tracking list
3. **File Deleted** → Watcher detects deletion → Removes from tracking list

### Benefits

- **10x faster** than full filesystem scans
- **Lower CPU usage** during active development
- **Immediate detection** of file changes
- **Better integration** with VSCode

### Debouncing

Changes are debounced (500ms delay) to prevent excessive processing during rapid file modifications.

## Project Type Examples

### Node.js / React Project

```bash
# 1. Create .knoxignore with Node.js preset
> Knox Checkpoint: Create .knoxignore
> Select: Node.js / TypeScript

# 2. Generated patterns include:
node_modules/
dist/
build/
.next/
package-lock.json
```

### Python / Django Project

```bash
# 1. Create .knoxignore with Python preset
> Knox Checkpoint: Create .knoxignore
> Select: Python

# 2. Generated patterns include:
__pycache__/
venv/
.pytest_cache/
*.pyc
db.sqlite3
```

### Rust Project

```bash
# 1. Create .knoxignore with Rust preset
> Knox Checkpoint: Create .knoxignore
> Select: Rust

# 2. Generated patterns include:
target/
Cargo.lock
*.rs.bk
```

## Best Practices

### For Small Projects (<1000 files)

✅ Use default settings  
✅ Create .knoxignore with appropriate preset  
✅ Let file watcher handle tracking  

### For Medium Projects (1000-10000 files)

✅ Set scan depth to 8-12  
✅ Use default file size limit (1MB)  
✅ Add project-specific ignore patterns  
✅ Monitor performance metrics  

### For Large Projects / Monorepos (>10000 files)

✅ Set scan depth to 6-8  
✅ Reduce file size limit to 512KB  
✅ Add comprehensive ignore patterns  
✅ Use multiple .knoxignore files in subdirectories  
✅ Monitor and optimize based on metrics  

## Troubleshooting

### Issue: Slow Performance

**Symptoms**: Checkpoints take >1 second to create

**Solutions**:
1. Reduce scan depth: `Knox Checkpoint: Configure Scan Depth`
2. Lower file size limit: `Knox Checkpoint: Configure Max File Size`
3. Add more ignore patterns to `.knoxignore`
4. Check metrics to identify bottlenecks

### Issue: Files Not Being Tracked

**Symptoms**: Changed files not appearing in checkpoints

**Solutions**:
1. Check `.knoxignore` patterns
2. Verify file extension is in tracked list
3. Check file size is under limit
4. Ensure file is in workspace directory

### Issue: Too Many Files Tracked

**Symptoms**: Checkpoints include too many files

**Solutions**:
1. Add ignore patterns for build outputs
2. Add ignore patterns for dependencies
3. Use preset for your project type
4. Review and customize `.knoxignore`

### Issue: File Watcher Not Working

**Symptoms**: Changes not detected in real-time

**Solutions**:
1. Check VSCode file watcher settings
2. Verify workspace path is valid
3. Restart VSCode
4. Check file extensions are tracked

## Storage Locations

### Debug Mode
```
<workspace>/.knox-debug/checkpoints/
```

### Production Mode
```
~/.knox/checkpoints/
```

You can open the storage location with:
```
Knox Checkpoint: Show Configuration
> Select: Storage Location
```

## Advanced Usage

### Programmatic API

```typescript
import { CheckpointManager } from './CheckpointManager';

const manager = CheckpointManager.getInstance();

// Configure
manager.setMaxScanDepth(12);
manager.setMaxFileSize(2 * 1024 * 1024);

// Create checkpoint
const checkpointId = await manager.createManualCheckpoint({
    description: 'Before refactoring',
    tags: ['refactor', 'safe-point']
});

// Get metrics
const metrics = manager.getPerformanceMetrics();
console.log(`Performance: ${metrics.averageScanDuration}ms avg`);

// Get configuration
const config = manager.getConfiguration();
console.log(`Current config:`, config);
```

### Custom Ignore Patterns

```typescript
import { generateKnoxIgnoreContent, getPresetByName } from './IgnorePatternPresets';

// Get preset
const preset = getPresetByName('Node.js / TypeScript');

// Generate content
const content = generateKnoxIgnoreContent(preset);

// Add custom patterns
const customContent = content + '\n# My custom patterns\nmy-custom-dir/\n';
```

## FAQ

### Q: Do I need Git installed?

**A**: No! The checkpoint system works completely independently of Git.

### Q: Can I use this with Git?

**A**: Yes! They work together seamlessly. Use Git for version control and checkpoints for AI session tracking.

### Q: How much disk space do checkpoints use?

**A**: Checkpoints are compressed and deduplicated. Typical usage: 50-100MB for small projects, 200-500MB for large projects.

### Q: Can I share checkpoints with my team?

**A**: Yes! Use export/import commands to share checkpoint backups.

### Q: What happens if I reach the storage limit?

**A**: Old checkpoints are automatically cleaned up based on retention settings (default: 7 days).

### Q: Can I restore to any checkpoint?

**A**: Yes! All checkpoints are fully restorable. You can also selectively restore individual files.

### Q: Does this work with all programming languages?

**A**: Yes! The system is language-agnostic and works with any text-based files.

### Q: What's the difference between checkpoints and Git commits?

**A**: 
- **Checkpoints**: AI-aware, automatic, session-based, fast, lightweight
- **Git Commits**: Manual, project-wide, permanent, team-shared

## Support & Resources

### Documentation
- [Standalone Mode Guide](./CHECKPOINT_STANDALONE.md)
- [Feature Enhancements](./CHECKPOINT_ENHANCEMENTS.md)
- [Implementation Summary](./IMPLEMENTATION_SUMMARY.md)
- [Migration Guide](./CHECKPOINT_MIGRATION_GUIDE.md)

### Getting Help
1. Check the troubleshooting section above
2. Review performance metrics for insights
3. Consult the detailed documentation
4. Open an issue on GitHub

## What's New in v2.0

✨ **Real-Time File Tracking** - Instant change detection with file watcher  
✨ **Smart Configuration** - User-friendly commands for all settings  
✨ **Project Presets** - Auto-generated .knoxignore for 10+ project types  
✨ **Performance Metrics** - Complete visibility into system performance  
✨ **Configurable Limits** - Scan depth and file size controls  

See [CHECKPOINT_ENHANCEMENTS.md](./CHECKPOINT_ENHANCEMENTS.md) for complete details.

---

**Made with ❤️ for AI-assisted development**

