pub mod router;
pub mod handlers;

// Re-export the main router function for easy access
pub use router::create_router;
