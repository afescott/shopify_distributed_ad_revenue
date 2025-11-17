# Auth Module - Shopify Margin Cost Dashboard

### What This Module Does
This authentication module provides **JWT-based authentication** for a multi-tenant Shopify merchant dashboard. It handles user login, role-based access control, and user management with a **viewer-only API** design.

### Database Schema
```sql
-- Core tables (auto-created by migrations)
merchants (id, shop_domain, shop_name, ...)
users (id, merchant_id, email, password_hash, role, is_active, ...)
```

### Role Hierarchy
- **Viewer** → Read-only access (Scope: Viewer)
- **Manager** → Edit products/orders (Scopes: Viewer, Manager)  
- **Admin** → Full control (Scopes: Viewer, Manager, Admin)

### Optional Environment Setup
```bash

export DATABASE_URL="postgres://exchange_user:exchange_password@localhost/exchange_api"
export PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n..."
export PUBLIC_KEY="-----BEGIN PUBLIC KEY-----\n..."
export JWT_EXPIRATION_HOURS="24"
```

### Quick Start Commands


#### 1. **Start the Server**
```bash
# Clone and navigate to auth module
cd auth_module

# Install dependencies and run (migrations run automatically)
cargo run

# Server starts on: http://localhost:8080
```

### **API Documentation**
- 📖 **Swagger UI:** http://localhost:8080/docs/

#### 2. **Create Test Users**
```bash
# Create test merchant and users
psql "postgres://exchange_user:exchange_password@localhost/exchange_api" \
  -f docs/create_test_user.sql

# This creates:
# - admin@test-shop.com (password: admin123) - Admin role
# - manager@test-shop.com (password: manager123) - Manager role  
# - viewer@test-shop.com (password: password) - Viewer role
# - inactive@test-shop.com (password: password) - Inactive account
```

