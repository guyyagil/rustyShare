# Routing Module Structure

This document describes the modular structure of the routing module in RustyShare.

## Overview

The routes have been refactored from a single large file into a modular structure for better organization, maintainability, and separation of concerns.

## Module Structure

```
src/server/
├── routing/                # Main routing module (renamed from routes)
│   ├── mod.rs              # Module exports and re-exports
│   ├── router.rs           # Main router configuration (37 lines - clean!)
│   ├── README.md           # This documentation
│   └── handlers/           # Handler modules
│       ├── mod.rs          # Handler exports and re-exports
│       ├── auth.rs         # Authentication-related handlers
│       ├── file_operations.rs # File management operations
│       ├── static_content.rs  # Static content serving
│       └── health.rs       # Health check endpoint
└── file_operations/        # File operation utilities
    ├── mod.rs              # Module exports
    └── streaming.rs        # HTTP streaming functionality (moved from server/)
```

## Module Responsibilities

### `mod.rs`
- Module exports and re-exports
- Makes the routes module accessible from the server

### `router.rs`
- Main router configuration
- Route definitions and middleware setup
- Entry point for the HTTP server (moved from routes.rs)

### `handlers/auth.rs`
- User authentication and authorization
- Login functionality
- Password protection for master page
- Cookie management

### `handlers/file_operations.rs`
- File upload, download, update, and deletion
- Folder creation and management
- File tree JSON API
- File streaming with range request support

### `handlers/static_content.rs`
- Serving static HTML content
- Generic static file handler

### `handlers/health.rs`
- Health check endpoint
- Simple status monitoring

### `../file_operations/streaming.rs`
- HTTP streaming functionality with range request support
- Efficient media streaming for large files
- Handles partial content requests (HTTP 206)

## Benefits of This Structure

1. **Separation of Concerns**: Each module has a specific responsibility
2. **Maintainability**: Easier to locate and modify specific functionality
3. **Testability**: Individual modules can be tested in isolation
4. **Readability**: Smaller, focused files are easier to understand
5. **Scalability**: New features can be added as separate modules

## Usage

The main `create_router()` function in `routes.rs` assembles all the handlers from the various modules into a complete Axum router. Each handler module exports only the functions it needs to expose, keeping implementation details private.

## Future Enhancements

- Add middleware modules for cross-cutting concerns
- Create separate modules for different API versions
- Add request/response type definitions in dedicated modules
- Implement error handling middleware
