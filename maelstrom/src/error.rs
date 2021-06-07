#[derive(Debug)]
pub struct MaelstromError {
    pub in_reply_to: i32,
    pub error: DefiniteError,
}
#[derive(Debug)]
pub struct DefiniteError {
    pub code: i32,
    pub text: String,
}

pub fn node_not_found(text: String) -> DefiniteError {
    DefiniteError {
        code: 1,
        text: text,
    }
}

pub fn not_supported(text: String) -> DefiniteError {
    DefiniteError {
        code: 10,
        text: text,
    }
}

pub fn temporarily_unavailable(text: String) -> DefiniteError {
    DefiniteError {
        code: 11,
        text: text,
    }
}

pub fn malformed_request(text: String) -> DefiniteError {
    DefiniteError {
        code: 12,
        text: text,
    }
}

pub fn abort(text: String) -> DefiniteError {
    DefiniteError {
        code: 14,
        text: text,
    }
}

pub fn key_does_not_exist(text: String) -> DefiniteError {
    DefiniteError {
        code: 20,
        text: text,
    }
}

pub fn key_already_exists(text: String) -> DefiniteError {
    DefiniteError {
        code: 21,
        text: text,
    }
}

pub fn precondition_failed(text: String) -> DefiniteError {
    DefiniteError {
        code: 22,
        text: text,
    }
}

pub fn txn_conflict(text: String) -> DefiniteError {
    DefiniteError {
        code: 30,
        text: text,
    }
}
