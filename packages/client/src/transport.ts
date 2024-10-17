// TODO: Redo this entire system when links are introduced
// TODO: Make this file work off Typescript types which are exported from Rust to ensure internal type-safety!
import { OperationType, RSPCError } from ".";
import { encode, decode } from "@msgpack/msgpack";

// TODO
export interface Transport {
  clientSubscriptionCallback?: (id: string, key: string, value: any) => void;

  doRequest(operation: OperationType, key: string, input: any): Promise<any>;
}

export const randomId = () => Math.random().toString(36).slice(2);

const timeouts = [1000, 2000, 5000, 10000]; // In milliseconds

export class WebsocketTransport implements Transport {
  private url: string;
  private ws: WebSocket;
  private requestMap = new Map<
    string,
    {
      op: unknown;
      cb: (data: any) => void;
    }
  >();
  clientSubscriptionCallback?: (id: string, value: any) => void;

  constructor(url: string) {
    this.url = url;
    this.ws = new WebSocket(url);
    this.attachEventListeners();
  }

  attachEventListeners() {
    // Resume all in-progress tasks
    for (const [_, item] of this.requestMap) {
      this.ws.send(encode(item.op));
    }

    this.ws.addEventListener("message", (event) => {
      const { id, result } = decode(event.data) as any;
      if (result.type === "event") {
        if (this.clientSubscriptionCallback)
          this.clientSubscriptionCallback(id, result.data);
      } else if (result.type === "response") {
        if (this.requestMap.has(id)) {
          this.requestMap
            .get(id)
            ?.cb({ type: "response", result: result.data });
          this.requestMap.delete(id);
        }
      } else if (result.type === "error") {
        const { message, code } = result.data;
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.cb({ type: "error", message, code });
          this.requestMap.delete(id);
        }
      } else {
        console.error(`Received event of unknown type '${result.type}'`);
      }
    });

    this.ws.addEventListener("close", (event) => {
      this.reconnect();
    });
  }

  async reconnect(timeoutIndex = 0) {
    let timeout =
      (timeouts[timeoutIndex] ?? timeouts[timeouts.length - 1]) +
      (Math.floor(Math.random() * 5000 /* 5 Seconds */) + 1);

    setTimeout(() => {
      let ws = new WebSocket(this.url);
      new Promise(function (resolve, reject) {
        ws.addEventListener("open", () => resolve(null));
        ws.addEventListener("close", reject);
      })
        .then(() => {
          this.ws = ws;
          this.attachEventListeners();
        })
        .catch((err) => this.reconnect(timeoutIndex++));
    }, timeout);
  }

  async doRequest(
    operation: OperationType,
    key: string,
    input: any,
    opts?: {
      id?: string;
    }
  ): Promise<any> {
    if (this.ws.readyState == 0) {
      let resolve: () => void;
      const promise = new Promise((res) => {
        resolve = () => res(undefined);
      });
      // @ts-ignore
      this.ws.addEventListener("open", resolve);
      await promise;
    }

    const id = randomId();
    let resolve: (data: any) => void;
    const promise = new Promise((res) => {
      resolve = res;
    });

    this.requestMap.set(id, {
      op: {
        id,
        method: operation,
        params: {
          path: key,
          input,
        },
      },
      // @ts-ignore
      cb: resolve,
    });

    this.ws.send(
      encode({
        id,
        method: operation,
        params: {
          path: key,
          input,
        },
      })
    );

    const body = (await promise) as any;
    if (body.type === "error") {
      const { code, message } = body;
      throw new RSPCError(code, message);
    } else if (body.type === "response") {
      return body.result;
    } else {
      throw new Error(
        `RSPC Websocket doRequest received invalid body type '${body?.type}'`
      );
    }
  }
}

// TODO
export class NoOpTransport implements Transport {
  constructor() { }

  async doRequest(
    operation: OperationType,
    key: string,
    input: string
  ): Promise<any> {
    return new Promise(() => { });
  }
}
