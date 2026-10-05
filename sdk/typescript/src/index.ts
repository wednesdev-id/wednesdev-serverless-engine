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

export function handle(req: Request): Response {
  return {
    status: 200,
    headers: [
      { name: "content-type", value: "text/plain" }
    ],
    body: new TextEncoder().encode("Hello from Wednes TypeScript SDK!")
  };
}
