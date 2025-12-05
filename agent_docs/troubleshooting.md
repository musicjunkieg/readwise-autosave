# Troubleshooting

## Common Issues

### Build Errors

**Issue:** Missing system dependencies for sqlx
```
Solution: Install OpenSSL dev packages
  macOS: brew install openssl
  Ubuntu: apt install libssl-dev pkg-config
```

**Issue:** Compile errors with async traits
```
Solution: Ensure async-trait is in dependencies
  cargo add async-trait
```

### Runtime Errors

**Issue:** Database connection fails
```
Solution: Check DATABASE_URL environment variable
  export DATABASE_URL="postgres://user:pass@localhost/dbname"
```

**Issue:** AT Protocol OAuth fails
```
Solution: Verify:
  1. Client ID is correctly formatted HTTPS URL
  2. Redirect URI matches configuration
  3. Required scopes are requested
```

### API Issues

**Issue:** Bluesky chat API returns 401
```
Solution: Include required header:
  AT-Protocol-Proxy: did:web:api.bsky.chat
```

**Issue:** Readwise rate limit hit
```
Solution: Implement exponential backoff
  Check Retry-After header in 429 response
```

---

## Learnings

(Append bug fixes and gotchas here as we encounter them)
