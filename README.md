# CLMM — Concentrated Liquidity Market Maker

A minimal concentrated liquidity AMM on Solana, built with [Anchor](https://www.anchor-lang.com/). Inspired by Uniswap V3.

## Architecture

```
programs/clmm/src/
├── instructions/
│   ├── initialize_pool.rs      # Create a new pool for a token pair
│   ├── open_position.rs        # Open a position within a tick range
│   ├── increase_liquidity.rs   # Add liquidity to an existing position
│   ├── decrease_liquidity.rs   # Remove liquidity from a position
│   ├── close_position.rs       # Close a position entirely
│   ├── swap.rs                 # Execute a token swap
│   └── shared_functions.rs     # Shared tick/liquidity helpers
├── states/
│   ├── pool.rs                 # Pool account (prices, liquidity, ticks)
│   ├── position.rs             # Per-user position (range + liquidity)
│   └── ticks.rs                # Tick arrays storing liquidity deltas
├── utils/
│   ├── math.rs                 # Core math (sqrt price, swap step, amounts)
│   └── constants.rs            # TICKS_PER_ARRAY, discriminator size
├── errors.rs
└── lib.rs                      # Program entrypoint
```

### Key Accounts

| Account | Seeds | Description |
|---|---|---|
| **Pool** | `["pool", token_0, token_1, tick_spacing]` | Stores global liquidity, current sqrt price, current tick, vault addresses |
| **TickArrayState** | `["tick_array", pool, start_tick]` | Array of `TICKS_PER_ARRAY` tick states, each tracking net/gross liquidity at that tick |
| **Position** | `["position", pool, owner, lower_tick, upper_tick]` | Tracks a user's liquidity within a specific tick range |

### Instruction Flow

```
initialize_pool → open_position → swap
                  ├── increase_liquidity
                  ├── decrease_liquidity
                  └── close_position
```

## Math

All prices are stored as **Q96 fixed-point** (`sqrt_price × 2^96`) to maintain precision without floating point.

### Price ↔ Tick

Each tick represents a 0.01% price change:

```
price = 1.0001^tick
sqrt_price_x96 = sqrt(1.0001^tick) × 2^96
```

`tick_to_sqrt_price_x96` uses precomputed Q64 constants for each bit of the tick (Uniswap V3 approach), then shifts to Q96.

### Token Amounts for Liquidity

Given a position `[tick_lower, tick_upper]` with liquidity `L` and current price `P`:

```
If P ≤ P_lower (all token_0):
    amount_0 = L × (√P_upper - √P_lower) / (√P_lower × √P_upper) × Q96

If P ≥ P_upper (all token_1):
    amount_1 = L × (√P_upper - √P_lower) / Q96

If P_lower < P < P_upper (both tokens):
    amount_0 = L × (√P_upper - √P) / (√P × √P_upper) × Q96
    amount_1 = L × (√P - √P_lower) / Q96
```

Uses **divide-before-multiply** to avoid u128 overflow: `L × delta / sqrt_a × Q96 / sqrt_b`.

### Swap Step

Each swap step computes how much can be traded within the current tick range:

```
amount_in  → delta_sqrt_price → new_sqrt_price → amount_out
```

The key formula (for a→b swaps, price decreasing):

```
delta_sqrt = amount_in × Q96 / (L + amount_in × sqrt_price / Q96)
new_sqrt   = sqrt_price - delta_sqrt
amount_out = L × (sqrt_price - new_sqrt) / Q96
```

When the swap crosses a tick boundary, liquidity is updated by the tick's `net_liquidity` delta.

## Running

```bash
anchor build
anchor test
```

## Tests

The integration test suite (`tests/clmm.ts`) covers:

1. **Pool initialization** — creates token pair, derives PDA, sets initial price at tick 0
2. **Open position** — deposits both tokens across `[-600, 60]` with 100k liquidity
3. **Swap (a→b)** — swaps 100 token_0 for ~99 token_1, verifies price movement
