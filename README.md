# sigma-accounting

[![CI](https://github.com/sigmatactical-org/accounting/actions/workflows/ci.yml/badge.svg)](https://github.com/sigmatactical-org/accounting/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.97.0-blue.svg)](https://www.rust-lang.org)

Accounting for Sigma Tactical Group. Stores scanned and digital bills, external integrations, and catalog-linked line items, with a server-rendered web UI and JSON API.

Repository: https://github.com/sigmatactical-org/accounting

Shared site chrome comes from [sigma-theme](https://github.com/sigmatactical-org/sigma-theme). Product SKUs are pulled from [sigma-catalog](https://github.com/sigmatactical-org/catalog) when configured.

## Features

- **Bills** — scanned (with scan URI) and digital bills with vendor, dates, status, and line items
- **Expenses** — categorized direct spend records with per-currency totals, optionally linked to a bill and/or a sales order
- **Receipts** — money received from customers, one row per successful [sigma-payments](https://github.com/sigmatactical-org/payments) charge; recorded by the cart at checkout and reconcilable against the payments charge log
- **Money in / out** — per-currency summary of receipts against expenses and paid bills
- **Integrations** — QuickBooks, Xero, and custom provider connections
- **Catalog linkage** — optional line-item references to catalog SKU ids
- **Order linkage** — optional bill-level reference to a [sigma-orders](https://github.com/sigmatactical-org/orders) sales order id, validated against the orders service when configured
- **Web UI** — browse, create, edit, and delete bills and integrations
- **JSON API** — programmatic CRUD for integration behind [sigma-identity](https://github.com/sigmatactical-org/identity)

## Configuration

| Variable | Purpose |
|----------|---------|
| `PORT` | Listen port (default `8080`) |
| `DATABASE_URL` | PostgreSQL connection URL (default `postgres://sigma:sigma@127.0.0.1:5432/sigma`) |
| `ACCOUNTING_CATALOG_BASE_URL` | Base URL of sigma-catalog (e.g. `http://127.0.0.1:8081/`) |
| `ACCOUNTING_ORDERS_BASE_URL` | Internal base URL of sigma-orders for `order_id` validation (e.g. `http://127.0.0.1:8085/`); unset skips validation |
| `ACCOUNTING_ORDERS_PUBLIC_URL` | Public base URL of the orders admin UI, for order links on the bills list |
| `ACCOUNTING_PAYMENTS_BASE_URL` | Internal base URL of sigma-payments (e.g. `http://127.0.0.1:8090/`), enabling receipt reconcile; unset hides the reconcile action |

## Data model

### Bills

Each bill has:

- `kind` — `scanned` or `digital`
- `status` — `draft`, `approved`, `paid`, or `void`
- `vendor`, optional `invoice_number`, `bill_date`, optional `due_date`
- optional `order_id` — linked sales order in sigma-orders (validated over HTTP when `ACCOUNTING_ORDERS_BASE_URL` is set; stored as an opaque reference, not a foreign key)
- `currency` — defaults to `USD`
- `line_items` — `[{ "sku_id"?, "description", "quantity", "unit_price_cents" }, …]`
- `total_cents` — computed from line items
- `scan_uri` — required for scanned bills (path or URL to the document)
- optional `notes`

### Expenses

Each expense has:

- `expense_date` — `YYYY-MM-DD`
- `category` — `materials`, `shipping`, `tooling`, `software`, `travel`, `fees`, or `other`
- `description` — required
- optional `vendor`
- `amount_cents` — at least 1
- `currency` — defaults to `USD`
- optional `receipt_uri` — path or URL to the receipt
- optional `bill_id` — the vendor bill this expense belongs to (validated against `accounting.bills`)
- optional `order_id` — linked sales order, validated like the bill-level `order_id`
- optional `notes`

### Receipts

Money in, one row per successful payments charge. Receipts are written by the
cart at checkout and by reconcile — there is no create/edit form.

- `charge_id` — the sigma-payments charge, unique across receipts and the
  idempotency key: recording the same charge twice is a no-op
- optional `order_id` — the sales order the payment was for
- `user_id` — the paying customer
- `kind` — `deposit`, `balance`, or `refund` (refunds subtract from money in)
- `amount_cents`, `currency` (defaults to `USD`), `occurred_at`
- optional `notes`

Charges reference the *cart*, not the order — the order does not exist yet
when the deposit is charged — so the cart is the only caller that can record a
receipt with both ids. Its push is best-effort and never fails a paid
checkout; **Reconcile with payments** on the index backfills anything missed
by sweeping `GET /api/charges` for successful charges without a receipt.

### Integrations

Each integration has:

- `name` — unique display name
- `provider` — `quickbooks`, `xero`, or `custom`
- `enabled` — boolean
- optional `external_account_id`, `webhook_url`, `notes`

## API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/bills` | List all bills |
| `GET` | `/bills/{id}` | Get one bill |
| `POST` | `/bills` | Create bill (JSON) |
| `PUT` | `/bills/{id}` | Update bill |
| `DELETE` | `/bills/{id}` | Delete bill |
| `GET` | `/expenses` | List all expenses |
| `GET` | `/expenses/{id}` | Get one expense |
| `POST` | `/expenses` | Create expense (JSON) |
| `PUT` | `/expenses/{id}` | Update expense |
| `DELETE` | `/expenses/{id}` | Delete expense |
| `GET` | `/integrations` | List integrations |
| `GET` | `/integrations/{id}` | Get one integration |
| `POST` | `/integrations` | Create integration |
| `PUT` | `/integrations/{id}` | Update integration |
| `DELETE` | `/integrations/{id}` | Delete integration |
| `GET` | `/receipts` | List all receipts |
| `GET` | `/receipts/{id}` | Get one receipt |
| `POST` | `/receipts` | Record a receipt (JSON); idempotent on `charge_id` — `201` when newly recorded, `200` with the existing receipt when the charge already had one |
| `DELETE` | `/receipts/{id}` | Delete receipt |
| `GET` | `/catalog/skus` | Proxy catalog SKUs (requires `ACCOUNTING_CATALOG_BASE_URL`) |

Example create digital bill:

```json
{
  "kind": "digital",
  "vendor": "Acme Corp",
  "invoice_number": "INV-100",
  "bill_date": "2026-01-15",
  "line_items": [
    {
      "sku_id": "<catalog-sku-uuid>",
      "description": "Widget",
      "quantity": 2,
      "unit_price_cents": 1500
    },
    {
      "description": "Shipping",
      "quantity": 1,
      "unit_price_cents": 500
    }
  ]
}
```

Example create integration:

```json
{
  "name": "QuickBooks Production",
  "provider": "quickbooks",
  "enabled": true,
  "external_account_id": "qb-123"
}
```

### Behind sigma-identity

Point identity at this service, for example:

```bash
IDENTITY_PROXY_TARGET=http://127.0.0.1:8080/
```

Browser clients call `/api/bills` on the identity host (with session + CSRF); identity forwards the request with a Bearer token attached.

## Development

```bash
cargo run -p sigma-accounting
```

Open http://localhost:8080

With catalog integration (run catalog on another port when accounting uses 8080):

```bash
# Terminal 1 — catalog
(cd sigma/it/catalog && PORT=8081 cargo run -p sigma-catalog)

# Terminal 2 — accounting (from sigma/it/accounting)
ACCOUNTING_CATALOG_BASE_URL=http://127.0.0.1:8081/ cargo run -p sigma-accounting
```

### Shared crates

`sigma-theme`, `sigma-pg`, and `sigma-config` are pinned git dependencies, so a
fresh clone builds with nothing but `cargo`: the revision in `Cargo.toml` is
fetched, and `build.rs` writes the `askama.toml` that points at sigma-theme's
templates wherever Cargo put them.

When one of those crates is checked out beside this repo and you are editing it,
link the checkouts so your edits are picked up without a push:

```bash
./scripts/prepare-local.sh
```

That writes `[patch]` entries into `.cargo/config.toml` (gitignored) for the
crates it finds and leaves the rest on their pinned revision; it prints what it
linked. Undo by deleting the file. Note that building against a linked checkout
rewrites `Cargo.lock` into path form — don't commit that; `platform`'s
`scripts/relock.sh` restores the git-resolved lockfile CI expects.

Bumping a shared crate is `platform/scripts/pin-shared-revs.sh <crate>` after
that crate is pushed, which updates every consumer's pin at once.

## Docker

Release is in **`.github/workflows/release.yml`** when configured. Locally:

```bash
./scripts/docker-build.sh
docker build -f Dockerfile build/image
```

Data is stored in the shared PostgreSQL `accounting` schema (`accounting.bills` and `accounting.integrations` JSONB table). Postgres runs in the [platform](https://github.com/sigmatactical-org/platform) kind stack — port-forward for local `cargo run`:

```bash
cd platform && ./scripts/postgres-dev.sh port-forward-bg && ./scripts/postgres-dev.sh migrate
```

## Brand & artwork

© Sigma Tactical Group. **All rights reserved.**

The Sigma Tactical Group name, logos, marks, artwork, and visual identity are **proprietary**. They are not covered by this repository's source-code license. See [BRANDING.md](BRANDING.md).

## License

MIT OR Apache-2.0 for **source code** only. Branding remains proprietary.
