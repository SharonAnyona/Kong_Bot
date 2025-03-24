import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Alert {
  'coin' : string,
  'target_price' : number,
  'user' : string,
}
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface Portfolio {
  'usd_balance' : number,
  'holdings' : Array<[string, number]>,
  'transactions' : Array<Transaction>,
}
export interface PriceHistory { 'last_price' : number }
export type Result = { 'Ok' : Portfolio } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : number } |
  { 'Err' : string };
export interface Transaction {
  'transaction_type' : TransactionType,
  'coin_id' : string,
  'timestamp' : bigint,
  'total_value' : number,
  'price' : number,
  'amount' : number,
}
export type TransactionType = { 'Buy' : null } |
  { 'Sell' : null };
export interface TransformArgs {
  'context' : Uint8Array | number[],
  'response' : HttpResponse,
}
export interface _SERVICE {
  'buy_cryptocurrency' : ActorMethod<[string, string, number], string>,
  'check_alerts' : ActorMethod<[], undefined>,
  'get_alerts' : ActorMethod<[], Array<[string, Alert]>>,
  'get_crypto_price_api' : ActorMethod<[string], string>,
  'get_icp_price' : ActorMethod<[], string>,
  'get_portfolio' : ActorMethod<[string], Result>,
  'get_portfolio_value' : ActorMethod<[string], Result_1>,
  'get_price_history' : ActorMethod<[], Array<[string, PriceHistory]>>,
  'get_supported_cryptocurrencies' : ActorMethod<[], Array<string>>,
  'initialize_portfolio' : ActorMethod<[string, [] | [number]], string>,
  'sell_cryptocurrency' : ActorMethod<[string, string, number], string>,
  'set_alert' : ActorMethod<[string, string, number], string>,
  'transform' : ActorMethod<[TransformArgs], HttpResponse>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
