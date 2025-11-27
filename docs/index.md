# gim Documentation

Welcome to the gim documentation. This is a fast, high-performance, modular system metrics and diagnostics CLI tool written in Rust.

## Table of Contents

- [API Documentation](api.md) - Detailed API reference
- [Architecture](architecture.md) - System architecture overview
- [Modules](modules.md) - Information about metric collection modules
- [Extending](extending.md) - How to add new modules and features

## About gim

gim (GENERIC INFRASTRUCTURE MONITOR) is designed to be:
- **Fast**: Written in Rust for high performance
- **Modular**: Easy to add new metric collectors
- **Flexible**: Multiple output formats (JSON, table, raw)
- **Extensible**: Clean trait-based architecture for easy extension

## Current Features

- CPU metric collection (usage, core count)
- Memory metric collection (usage, available/free/used, swap)
- Unified MetricCollector trait for easy module additions
- Multiple output formats (JSON, table, raw)
- CLI argument parsing with clap

## Planned Features

- Disk and network metric collectors
- TUI dashboard implementation
- Configuration file support
- Live monitoring capabilities