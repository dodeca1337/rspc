         // This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

         export const commands = {

}

export const events = __makeEvents__<{
msg: Msg,
transportResp: Response[]
}>({
msg: "plugin:rspc:msg",
transportResp: "plugin:rspc:transport-resp"
})

/** user-defined types **/

export type Error = { code: ErrorCode; message: string }
/**
 * TODO
 */
export type ErrorCode = "BadRequest" | "Unauthorized" | "Forbidden" | "NotFound" | "Timeout" | "Conflict" | "PreconditionFailed" | "PayloadTooLarge" | "MethodNotSupported" | "ClientClosedRequest" | "InternalServerError"
export type Msg = any
export type ProcedureError = { Exec: Error } | { Resolver: any }
/**
 * The type of a response from rspc.
 * 
 * @internal
 */
export type Response = (
/**
 * The result of a successful operation.
 */
{ type: "value"; value: any } | 
/**
 * The result of a failed operation.
 */
{ type: "error"; value: ProcedureError } | 
/**
 * A message to indicate that the operation is complete.
 */
{ type: "complete" }) & { id: number }
/**
 * A value that can be a successful result or an error.
 * 
 * @internal
 */
export type ResponseInner = 
/**
 * The result of a successful operation.
 */
{ type: "value"; value: any } | 
/**
 * The result of a failed operation.
 */
{ type: "error"; value: ProcedureError } | 
/**
 * A message to indicate that the operation is complete.
 */
{ type: "complete" }

/** tauri-specta globals **/

         import { invoke as TAURI_INVOKE } from "@tauri-apps/api";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindowHandle as __WebviewWindowHandle__ } from "@tauri-apps/api/window";

type __EventObj__<T> = {
  listen: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
  once: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
  emit: T extends null
    ? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
    : (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

type __Result__<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
  mappings: Record<keyof T, string>
) {
  return new Proxy(
    {} as unknown as {
      [K in keyof T]: __EventObj__<T[K]> & {
        (handle: __WebviewWindowHandle__): __EventObj__<T[K]>;
      };
    },
    {
      get: (_, event) => {
        const name = mappings[event as keyof T];

        return new Proxy((() => {}) as any, {
          apply: (_, __, [window]: [__WebviewWindowHandle__]) => ({
            listen: (arg: any) => window.listen(name, arg),
            once: (arg: any) => window.once(name, arg),
            emit: (arg: any) => window.emit(name, arg),
          }),
          get: (_, command: keyof __EventObj__<any>) => {
            switch (command) {
              case "listen":
                return (arg: any) => TAURI_API_EVENT.listen(name, arg);
              case "once":
                return (arg: any) => TAURI_API_EVENT.once(name, arg);
              case "emit":
                return (arg: any) => TAURI_API_EVENT.emit(name, arg);
            }
          },
        });
      },
    }
  );
}

     