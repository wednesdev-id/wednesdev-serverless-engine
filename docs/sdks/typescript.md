# TypeScript / JavaScript SDK

The **Wednes TypeScript & JavaScript SDK** enables web developers to write serverless functions using familiar modern TypeScript or ES modules, compiled directly into WASI WebAssembly components via [ComponentizeJS](https://github.com/bytecodealliance/ComponentizeJS).

---

## Type Definitions

The TypeScript interface directly mirrors the `wednes:function@0.1.0` ABI:

```typescript
export interface Header {
  name: string;
  value: string;
}

export interface Request {
  method: string;
  path: string;
  headers: Header[];
  body: Uint8Array;
}

export interface Response {
  status: number;
  headers: Header[];
  body: Uint8Array;
}
```

Every function exports a single root `handle` function:

```typescript
export function handle(req: Request): Response {
  // Application logic here
}
```

---

## Examples

### 1. Minimal JSON Greeting

```typescript
import { Request, Response } from "./types";

export function handle(req: Request): Response {
  const payload = {
    status: 200,
    message: "Hello from Wednes Serverless TypeScript!",
    timestamp: new Date().toISOString()
  };

  const bodyBytes = new TextEncoder().encode(JSON.stringify(payload));

  return {
    status: 200,
    headers: [{ name: "content-type", value: "application/json" }],
    body: bodyBytes
  };
}
```

---

### 2. WhatsApp CRM Webhook Handler

Handles both webhook verification challenge (GET) and incoming message dispatch (POST):

```typescript
import { Request, Response } from "./types";

export function handle(req: Request): Response {
  const method = req.method.toUpperCase();

  // 1. WhatsApp Verification Challenge (GET)
  if (method === "GET") {
    return {
      status: 200,
      headers: [{ name: "content-type", value: "text/plain" }],
      body: new TextEncoder().encode("CHALLENGE_ACCEPTED")
    };
  }

  // 2. Inbound Message Processing (POST)
  if (method === "POST") {
    try {
      const rawText = new TextDecoder().decode(req.body);
      const data = JSON.parse(rawText);

      const sender = data.entry?.[0]?.changes?.[0]?.value?.messages?.[0]?.from || "anonymous";
      const messageText = data.entry?.[0]?.changes?.[0]?.value?.messages?.[0]?.text?.body || "";

      const replyPayload = {
        messaging_product: "whatsapp",
        to: sender,
        text: {
          body: `Thank you for contacting WednesDev! We received: "${messageText}". An agent will respond shortly.`
        }
      };

      return {
        status: 200,
        headers: [{ name: "content-type", value: "application/json" }],
        body: new TextEncoder().encode(JSON.stringify(replyPayload))
      };
    } catch (err: any) {
      return {
        status: 400,
        headers: [{ name: "content-type", value: "application/json" }],
        body: new TextEncoder().encode(JSON.stringify({ error: err.message }))
      };
    }
  }

  return {
    status: 405,
    headers: [{ name: "allow", value: "GET, POST" }],
    body: new TextEncoder().encode("Method Not Allowed")
  };
}
```

---

## Build Pipeline (`package.json`)

Configure your project to build into a standalone component:

```json
{
  "name": "my-ts-function",
  "version": "1.0.0",
  "scripts": {
    "build": "tsc && npx componentize-js dist/index.js -w ../../wit -o function.wasm"
  },
  "devDependencies": {
    "@bytecodealliance/componentize-js": "^0.13.0",
    "typescript": "^5.4.0"
  }
}
```
