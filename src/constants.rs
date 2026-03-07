//! Constants for WebSocket client

/// Default heartbeat interval in seconds
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 30;

/// Maximum reconnection attempts
pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// WebSocket URLs
pub const PRODUCTION_WS_URL: &str = "wss://www.deribit.com/ws/api/v2";
/// WebSocket URL for Deribit testnet
pub const TESTNET_WS_URL: &str = "wss://test.deribit.com/ws/api/v2";

/// JSON-RPC methods
pub mod methods {
    // Authentication
    /// Public authentication method
    pub const PUBLIC_AUTH: &str = "public/auth";
    /// Private logout method
    pub const PRIVATE_LOGOUT: &str = "private/logout";

    // Subscriptions
    /// Public subscription method
    pub const PUBLIC_SUBSCRIBE: &str = "public/subscribe";
    /// Public unsubscription method
    pub const PUBLIC_UNSUBSCRIBE: &str = "public/unsubscribe";
    /// Public unsubscribe from all channels
    pub const PUBLIC_UNSUBSCRIBE_ALL: &str = "public/unsubscribe_all";
    /// Private subscription method
    pub const PRIVATE_SUBSCRIBE: &str = "private/subscribe";
    /// Private unsubscription method
    pub const PRIVATE_UNSUBSCRIBE: &str = "private/unsubscribe";
    /// Private unsubscribe from all channels
    pub const PRIVATE_UNSUBSCRIBE_ALL: &str = "private/unsubscribe_all";

    // Market data
    /// Get ticker information
    pub const PUBLIC_GET_TICKER: &str = "public/ticker";
    /// Get order book data
    pub const PUBLIC_GET_ORDERBOOK: &str = "public/get_order_book";
    /// Get trade history
    pub const PUBLIC_GET_TRADES: &str = "public/get_last_trades_by_instrument";
    /// Get instrument information
    pub const PUBLIC_GET_INSTRUMENTS: &str = "public/get_instruments";

    // Trading
    /// Place buy order
    pub const PRIVATE_BUY: &str = "private/buy";
    /// Place sell order
    pub const PRIVATE_SELL: &str = "private/sell";
    /// Cancel specific order
    pub const PRIVATE_CANCEL: &str = "private/cancel";
    /// Cancel all orders
    pub const PRIVATE_CANCEL_ALL: &str = "private/cancel_all";
    /// Cancel all orders by currency
    pub const PRIVATE_CANCEL_ALL_BY_CURRENCY: &str = "private/cancel_all_by_currency";
    /// Cancel all orders by instrument
    pub const PRIVATE_CANCEL_ALL_BY_INSTRUMENT: &str = "private/cancel_all_by_instrument";
    /// Edit an existing order
    pub const PRIVATE_EDIT: &str = "private/edit";
    /// Get open orders
    pub const PRIVATE_GET_OPEN_ORDERS: &str = "private/get_open_orders";

    // Account
    /// Get account summary
    pub const PRIVATE_GET_ACCOUNT_SUMMARY: &str = "private/get_account_summary";
    /// Get positions
    pub const PRIVATE_GET_POSITIONS: &str = "private/get_positions";
    /// Get subaccounts
    pub const PRIVATE_GET_SUBACCOUNTS: &str = "private/get_subaccounts";
    /// Get order state
    pub const PRIVATE_GET_ORDER_STATE: &str = "private/get_order_state";
    /// Get order history by currency
    pub const PRIVATE_GET_ORDER_HISTORY_BY_CURRENCY: &str = "private/get_order_history_by_currency";

    // Position management
    /// Close an existing position
    pub const PRIVATE_CLOSE_POSITION: &str = "private/close_position";
    /// Move positions between subaccounts
    pub const PRIVATE_MOVE_POSITIONS: &str = "private/move_positions";

    // Test
    /// Test connection
    pub const PUBLIC_TEST: &str = "public/test";
    /// Get server time
    pub const PUBLIC_GET_TIME: &str = "public/get_time";
    /// Hello message
    pub const PUBLIC_HELLO: &str = "public/hello";
}

/// Subscription channels
pub mod channels {
    /// Ticker channel
    pub const TICKER: &str = "ticker";
    /// Order book channel
    pub const ORDERBOOK: &str = "book";
    /// Trades channel
    pub const TRADES: &str = "trades";
    /// User orders channel
    pub const USER_ORDERS: &str = "user.orders";
    /// User trades channel
    pub const USER_TRADES: &str = "user.trades";
    /// User portfolio channel
    pub const USER_PORTFOLIO: &str = "user.portfolio";
    /// Incremental ticker channel
    pub const INCREMENTAL_TICKER: &str = "incremental_ticker";
    /// Price ranking channel
    pub const PRICE_RANKING: &str = "deribit_price_ranking";
    /// Price statistics channel
    pub const PRICE_STATISTICS: &str = "deribit_price_statistics";
    /// Volatility index channel
    pub const VOLATILITY_INDEX: &str = "deribit_volatility_index";
    /// Platform state channel
    pub const PLATFORM_STATE: &str = "platform_state";
    /// Platform state public methods channel
    pub const PLATFORM_STATE_PUBLIC_METHODS: &str = "platform_state.public_methods_state";
    /// Instrument state channel
    pub const INSTRUMENT_STATE: &str = "instrument.state";
    /// Perpetual channel
    pub const PERPETUAL: &str = "perpetual";
    /// Mark price options channel
    pub const MARKPRICE_OPTIONS: &str = "markprice.options";
    /// Quote channel
    pub const QUOTE: &str = "quote";
    /// Block RFQ trades channel
    pub const BLOCK_RFQ_TRADES: &str = "block_rfq.trades";
    /// Block trade confirmations channel
    pub const BLOCK_TRADE_CONFIRMATIONS: &str = "block_trade_confirmations";
    /// User MMP trigger channel
    pub const USER_MMP_TRIGGER: &str = "user.mmp_trigger";
    /// User access log channel
    pub const USER_ACCESS_LOG: &str = "user.access_log";
    /// User lock channel
    pub const USER_LOCK: &str = "user.lock";
}
