# SPA Routing Fix for smokeping-rs

## Problem
When users directly accessed specific target URLs in the browser (e.g., typing `http://localhost:3000/targets/1` directly in the address bar), the application showed a 404 error instead of displaying the target details page correctly.

This is a common Single Page Application (SPA) routing problem where:
1. Vue.js router handles client-side navigation correctly when users click through the app
2. But direct URL access fails because the backend server doesn't know about frontend routes
3. The server needs to be configured to serve the main index.html file for all non-API routes

## Solution
Implemented a custom fallback handler that:
1. **Preserves API functionality**: Routes starting with `/api/` continue to work normally and return proper 404 errors for non-existent endpoints
2. **Enables SPA routing**: All other routes serve the Vue.js `index.html` file with a 200 status code
3. **Maintains WebSocket support**: The `/ws` route continues to work for real-time updates

## Changes Made

### 1. Updated imports in `src/main.rs`
```rust
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
```

### 2. Added custom SPA fallback handler
```rust
// Custom fallback handler for SPA routing
async fn spa_fallback(request: Request) -> impl IntoResponse {
    let path = request.uri().path();
    
    // If the request is for an API route, return 404
    if path.starts_with("/api/") {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }
    
    // For all other routes, serve the index.html file from embedded assets
    if let Some(content) = Frontend::get("index.html") {
        use axum::response::Html;
        Html(content.data).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "SPA file not found").into_response()
    }
}
```

### 3. Updated router configuration
```rust
let app = Router::new()
    .nest("/api", api_router)
    .route("/ws", get(ws_handler))
    .fallback(spa_fallback)  // Use custom fallback instead of ServeEmbed
    .layer(cors)
    .with_state(state);
```

## Testing Results
All tests pass successfully:

✅ **API Routes**: `/api/targets` returns 200 with JSON data  
✅ **API 404s**: `/api/nonexistent` returns 404 (not SPA content)  
✅ **SPA Routes**: `/targets/1`, `/targets/999` return 200 with HTML  
✅ **Root Route**: `/` returns 200 with HTML  
✅ **Direct Access**: Typing URLs directly in browser works correctly  

## Benefits
1. **Fixed SPA routing**: Users can now bookmark and directly access any frontend route
2. **Preserved API integrity**: API routes maintain proper HTTP status codes
3. **No breaking changes**: Existing functionality remains unchanged
4. **Better user experience**: No more 404 errors when refreshing or sharing URLs

## Usage
After this fix:
- ✅ Direct browser access to `http://localhost:3000/targets/1` works
- ✅ Refreshing the page on any frontend route works
- ✅ Sharing URLs to specific pages works
- ✅ API endpoints continue to work normally
- ✅ WebSocket connections continue to work for real-time updates
