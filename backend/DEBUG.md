# Debugging Guide

## Running with Full Debug Logs

The backend is now configured with debug logging. To see detailed error messages:

### 1. Start the Backend

```bash
cd backend
cargo run
```

You should now see detailed logs including:
- HTTP requests/responses (via actix-web Logger middleware)
- Application debug logs (via env_logger)
- Tracing logs (via tracing-subscriber)

### 2. Test the /strategies/generate Endpoint

From another terminal, test with curl:

```bash
curl -X POST http://localhost:8080/strategies/generate \
  -H "Content-Type: application/json" \
  -d '{
    "symbols": ["BTCUSDT"],
    "intervals": ["1h"],
    "top_n": 1
  }'
```

### 3. Check the Backend Logs

Look for error messages in the backend terminal. Common issues:

#### Binance API Rate Limits
- Error: "Too many requests"
- Solution: Add delays between requests or use API keys

#### Missing Environment Variables
- Error: "BINANCE_API_KEY not set"
- Solution: API keys are optional for public data, should work without them

#### Database Connection
- Error: "Failed to connect to DB"
- Solution: Ensure docker-compose is running: `docker-compose up -d`

#### Data Fetching Issues
- Error: "Symbol not found" or "Invalid interval"
- Solution: Use valid Binance symbols (BTCUSDT, ETHUSDT, etc.) and intervals (1h, 4h, 12h)

### 4. Watch Backend Logs in Real-Time

The logs will show:
- `[INFO]` - General application flow
- `[DEBUG]` - Detailed operation logs
- `[ERROR]` - Errors with stack traces

Look for lines containing "generate_strategies" or "strategy_generator" to see what's happening.

## Common Errors and Solutions

### 500 Internal Server Error

If you see this, check the backend logs for the actual error. Common causes:

1. **Binance rate limiting**: Try reducing the number of symbols/intervals
2. **Network issues**: Ensure you can reach api.binance.com
3. **Invalid symbols**: Some pairs like FDUSD pairs might not exist or have limited data
4. **Optimizer issues**: Check if the krypto library is properly compiled

### Recommended Test Configuration

Start small to test if everything works:

```bash
# Test with just 1 symbol and 1 interval
curl -X POST http://localhost:8080/strategies/generate \
  -H "Content-Type: application/json" \
  -d '{
    "symbols": ["BTCUSDT"],
    "intervals": ["1h"],
    "top_n": 1
  }'
```

If this works, gradually increase:
- 2-3 symbols
- 2-3 intervals
- More strategies (top_n)

## Example Expected Output

Success (200):
```json
{
  "message": "Generation complete",
  "strategies_created": 3
}
```

Error (500):
```json
{
  "error": "Strategy error: Failed to fetch data for BTCFDUSD 1h: Symbol not found"
}
```
