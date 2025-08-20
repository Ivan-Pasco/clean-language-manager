# Clean Language Manager Documentation

## Overview

This directory contains comprehensive documentation for the Clean Language Manager (`cleanmanager`), a Rust-based version manager for the Clean Language compiler.

## Documentation Structure

### 📋 [Functional Specification](functional-specification.md)
Complete functional requirements and behavior specification including:
- Core functionality overview
- Version installation and management
- Command specifications
- Platform support details
- Integration requirements

### 🏗️ [Architecture Guide](architecture.md)
Technical architecture and implementation details including:
- High-level system design
- Component relationships
- Data flow diagrams
- Directory structure
- Security model
- Performance characteristics

### 📚 [API Reference](api-reference.md)
Complete command-line interface documentation including:
- All commands with syntax and examples
- Configuration file formats
- Environment variables
- Exit codes
- Error messages

### 👤 [User Guide](user-guide.md)
Practical usage guide for developers including:
- Quick start instructions
- Common workflows
- Best practices
- Troubleshooting guide
- Migration instructions

## Quick Navigation

### For New Users
1. Start with [User Guide](user-guide.md) for installation and basic usage
2. Reference [API Reference](api-reference.md) for specific commands
3. Check [User Guide - Troubleshooting](user-guide.md#troubleshooting) if you encounter issues

### For Developers
1. Review [Architecture Guide](architecture.md) for system design
2. See [Functional Specification](functional-specification.md) for requirements
3. Use [API Reference](api-reference.md) for implementation details

### For Contributors
1. Understand [Architecture Guide](architecture.md) for code organization
2. Follow [Functional Specification](functional-specification.md) for feature requirements
3. Reference existing patterns in the codebase

## Key Concepts

### Version Management
The manager handles multiple compiler versions in isolated directories, allowing seamless switching between versions for different projects.

### Shim System
A lightweight proxy (`~/.cleanmanager/bin/cln`) that routes `cln` commands to the appropriate version based on global and project-specific settings.

### Project Integration
Project-specific versions are managed via `.cleanlanguage/.cleanversion` files that override global settings within project directories.

## Documentation Maintenance

### Updating Documentation
When modifying the codebase:
1. Update relevant documentation sections
2. Ensure examples remain accurate
3. Update command output samples if CLI changes
4. Verify cross-references between documents

### Documentation Standards
- Use clear, concise language
- Include practical examples
- Maintain consistent formatting
- Keep diagrams and code samples up-to-date