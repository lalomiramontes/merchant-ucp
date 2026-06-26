---
name: ucp-buyer
description: Buy a product from a merchant that implements the Universal Commerce Protocol (UCP). Use this skill whenever asked to purchase, order, or check out items from a UCP merchant server.
---

# UCP Buyer

Acts as an autonomous buyer against a merchant server that implements the
Universal Commerce Protocol (UCP), spec version `2026-04-08`.

## When to use this skill

Use this whenever asked to buy, order, or check out one or more items from a
merchant that exposes a UCP discovery document. The merchant's base URL will
be given to you (e.g. `http://localhost:3000`).

## Important: do not guess endpoint paths

UCP endpoint paths are NOT predictable from REST naming conventions alone.
Do not try variations like `/checkout`, `/checkout/create`, `/sessions`, etc.
The exact paths are fixed and listed below — use them exactly as written.

## Step-by-step procedure

### 1. Discover the merchant

```
GET {base_url}/.well-known/ucp
```

This returns a JSON document with this shape:

```json
{
  "ucp": {
    "version": "2026-04-08",
    "services": {
      "dev.ucp.shopping": [
        { "version": "2026-04-08", "transport": "rest", "endpoint": "{base_url}/ucp/v1" }
      ]
    },
    "payment_handlers": {
      "<handler_namespace>": [{ "id": "<handler_id>", "version": "2026-04-08" }]
    }
  }
}
```

Extract:
- The `endpoint` value under `services.dev.ucp.shopping[0]` — this is your
  checkout API base path (e.g. `{base_url}/ucp/v1`). Call it `checkout_base`.
- The `id` of the first entry under `payment_handlers` — you will need this
  in step 3. Call it `payment_handler_id`.

### 2. Create a checkout session

```
POST {checkout_base}/checkout-sessions
Content-Type: application/json

{
  "line_items": [
    { "id": "<item id>", "title": "<item title>", "quantity": <int>, "unit_price": <int, smallest currency unit>, "currency": "<ISO code>" }
  ],
  "buyer": { "name": "<buyer name>", "email": "<buyer email>" }
}
```

The response includes an `id` field (e.g. `chk_xxxxx`) — this is the
checkout session ID. Call it `checkout_id`. The response also includes
`status`, which will be `"incomplete"` at this point — that is expected.

### 3. Attach a payment handler

```
PUT {checkout_base}/checkout-sessions/{checkout_id}
Content-Type: application/json

{
  "payment_handler_id": "<payment_handler_id from step 1>"
}
```

Check the `status` field in the response:
- `"ready_for_complete"` — proceed to step 4.
- `"incomplete"` — something required is still missing (commonly a buyer
  email). Re-check the buyer object you sent in step 2 and retry this step
  with corrected data.

### 4. Complete the checkout

```
POST {checkout_base}/checkout-sessions/{checkout_id}/complete
```

- HTTP 200 with `"status": "completed"` means the purchase succeeded.
- HTTP 409 means the checkout was not in `ready_for_complete` state. Go back
  to step 3 and confirm the payment handler was attached correctly before
  retrying.

## Summary of fixed paths

| Step              | Method | Path                                          |
|-------------------|--------|------------------------------------------------|
| Discover merchant | GET    | `/.well-known/ucp`                              |
| Create checkout   | POST   | `{checkout_base}/checkout-sessions`             |
| Update checkout   | PUT    | `{checkout_base}/checkout-sessions/{id}`        |
| Complete checkout | POST   | `{checkout_base}/checkout-sessions/{id}/complete` |
| Cancel checkout   | POST   | `{checkout_base}/checkout-sessions/{id}/cancel`   |

Always report back the final checkout JSON (id, status, total, currency) so
the result can be verified.
