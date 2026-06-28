# sigma-accounting

Accounting for Sigma Tactical Group. Stores scanned and digital bills, external integrations, and catalog-linked line items, with a server-rendered web UI and JSON API.

Repository: https://github.com/sigmatactical-org/accounting

Shared site chrome comes from [sigma-theme](https://github.com/sigmatactical-org/sigma-theme). Product SKUs are pulled from [sigma-catalog](https://github.com/sigmatactical-org/catalog) when configured.

## Features

- **Bills** — scanned (with scan URI) and digital bills with vendor, dates, status, and line items
- **Integrations** — QuickBooks, Xero, and custom provider connections
- **Catalog linkage** — optional line-item references to catalog SKU ids
- **Web UI** — browse, create, edit, and delete bills and integrations
- **JSON API** — programmatic CRUD for integration behind [sigma-identity](https://github.com/sigmatactical-org/identity)

## Configuration

| Variable | Purpose |
|----------|---------|
| `PORT` | Listen port (default `8080`) |
| `ACCOUNTING_DATA_PATH` | JSON database path (default `data/accounting.json`) |
| `ACCOUNTING_CATALOG_BASE_URL` | Base URL of sigma-catalog (e.g. `http://127.0.0.1:8081/`) |

## Data model

### Bills

Each bill has:

- `kind` — `scanned` or `digital`
- `status` — `draft`, `approved`, `paid`, or `void`
- `vendor`, optional `invoice_number`, `bill_date`, optional `due_date`
- `currency` — defaults to `USD`
- `line_items` — `[{ "sku_id"?, "description", "quantity", "unit_price_cents" }, …]`
- `total_cents` — computed from line items
- `scan_uri` — required for scanned bills (path or URL to the document)
- optional `notes`

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
| `GET` | `/integrations` | List integrations |
| `GET` | `/integrations/{id}` | Get one integration |
| `POST` | `/integrations` | Create integration |
| `PUT` | `/integrations/{id}` | Update integration |
| `DELETE` | `/integrations/{id}` | Delete integration |
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
./scripts/prepare-local.sh
cargo run -p sigma-accounting
```

Open http://localhost:8080

With catalog integration:

```bash
ACCOUNTING_CATALOG_BASE_URL=http://127.0.0.1:8081/ cargo run -p sigma-accounting
```

## Docker

Release is in **`.github/workflows/release.yml`** when configured. Locally:

```bash
./scripts/docker-build.sh
docker build -f Dockerfile build/image
```

Mount a volume at `/app/data` (or set `ACCOUNTING_DATA_PATH`) so accounting data persists across restarts.

## License

MIT OR Apache-2.0
