use super::ErrorCodes;

#[test]
fn exposes_every_json_rpc_and_lsp_error_code() {
    assert_eq!(ErrorCodes::PARSE_ERROR, -32700);
    assert_eq!(ErrorCodes::INVALID_REQUEST, -32600);
    assert_eq!(ErrorCodes::METHOD_NOT_FOUND, -32601);
    assert_eq!(ErrorCodes::INVALID_PARAMS, -32602);
    assert_eq!(ErrorCodes::INTERNAL_ERROR, -32603);
    assert_eq!(ErrorCodes::JSONRPC_RESERVED_ERROR_RANGE_START, -32099);
    assert_eq!(ErrorCodes::SERVER_ERROR_START, -32099);
    assert_eq!(ErrorCodes::SERVER_NOT_INITIALIZED, -32002);
    assert_eq!(ErrorCodes::UNKNOWN_ERROR_CODE, -32001);
    assert_eq!(ErrorCodes::JSONRPC_RESERVED_ERROR_RANGE_END, -32000);
    assert_eq!(ErrorCodes::SERVER_ERROR_END, -32000);
    assert_eq!(ErrorCodes::LSP_RESERVED_ERROR_RANGE_START, -32899);
    assert_eq!(ErrorCodes::REQUEST_FAILED, -32803);
    assert_eq!(ErrorCodes::SERVER_CANCELLED, -32802);
    assert_eq!(ErrorCodes::CONTENT_MODIFIED, -32801);
    assert_eq!(ErrorCodes::REQUEST_CANCELLED, -32800);
    assert_eq!(ErrorCodes::LSP_RESERVED_ERROR_RANGE_END, -32800);
}

#[test]
fn preserves_source_aliases_and_equal_value_boundaries() {
    assert_eq!(
        ErrorCodes::SERVER_ERROR_START,
        ErrorCodes::JSONRPC_RESERVED_ERROR_RANGE_START
    );
    assert_eq!(
        ErrorCodes::SERVER_ERROR_END,
        ErrorCodes::JSONRPC_RESERVED_ERROR_RANGE_END
    );
    assert_eq!(
        ErrorCodes::REQUEST_CANCELLED,
        ErrorCodes::LSP_RESERVED_ERROR_RANGE_END
    );
}

#[test]
fn exposes_signed_integer_codes() {
    let parse_error: i64 = ErrorCodes::PARSE_ERROR;
    let request_cancelled: i64 = ErrorCodes::REQUEST_CANCELLED;

    assert_eq!((parse_error, request_cancelled), (-32700, -32800));
}
