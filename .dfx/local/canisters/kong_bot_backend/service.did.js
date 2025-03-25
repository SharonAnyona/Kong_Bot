export const idlFactory = ({ IDL }) => {
  const Alert = IDL.Record({
    'coin' : IDL.Text,
    'target_price' : IDL.Float64,
    'user' : IDL.Text,
  });
  const TransactionType = IDL.Variant({ 'Buy' : IDL.Null, 'Sell' : IDL.Null });
  const Transaction = IDL.Record({
    'transaction_type' : TransactionType,
    'coin_id' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'total_value' : IDL.Float64,
    'price' : IDL.Float64,
    'amount' : IDL.Float64,
  });
  const Portfolio = IDL.Record({
    'usd_balance' : IDL.Float64,
    'holdings' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Float64)),
    'transactions' : IDL.Vec(Transaction),
  });
  const Result = IDL.Variant({ 'Ok' : Portfolio, 'Err' : IDL.Text });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Float64, 'Err' : IDL.Text });
  const PriceHistory = IDL.Record({ 'last_price' : IDL.Float64 });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const TransformArgs = IDL.Record({
    'context' : IDL.Vec(IDL.Nat8),
    'response' : HttpResponse,
  });
  return IDL.Service({
    'buy_cryptocurrency' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Float64],
        [IDL.Text],
        [],
      ),
    'check_alerts' : IDL.Func([], [], []),
    'get_alerts' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, Alert))],
        ['query'],
      ),
    'get_crypto_price_api' : IDL.Func([IDL.Text], [IDL.Text], []),
    'get_icp_price' : IDL.Func([], [IDL.Text], []),
    'get_portfolio' : IDL.Func([IDL.Text], [Result], ['query']),
    'get_portfolio_value' : IDL.Func([IDL.Text], [Result_1], []),
    'get_price_history' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, PriceHistory))],
        ['query'],
      ),
    'get_supported_cryptocurrencies' : IDL.Func(
        [],
        [IDL.Vec(IDL.Text)],
        ['query'],
      ),
    'initialize_portfolio' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Float64)],
        [IDL.Text],
        [],
      ),
    'sell_cryptocurrency' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Float64],
        [IDL.Text],
        [],
      ),
    'set_alert' : IDL.Func([IDL.Text, IDL.Text, IDL.Float64], [IDL.Text], []),
    'transform' : IDL.Func([TransformArgs], [HttpResponse], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
