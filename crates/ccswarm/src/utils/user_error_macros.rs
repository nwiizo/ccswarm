/// User-facing error macros for better error messages

/// Create a user-friendly error message
#[macro_export]
macro_rules! user_error {
    ($msg:expr) => {
        $crate::error::CCSwarmError::UserError($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::CCSwarmError::UserError(format!($fmt, $($arg)*))
    };
}

/// Create a user-friendly error with context
#[macro_export]
macro_rules! user_error_context {
    ($context:expr, $msg:expr) => {
        $crate::error::CCSwarmError::UserError(format!("{}: {}", $context, $msg))
    };
    ($context:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::error::CCSwarmError::UserError(format!("{}: {}", $context, format!($fmt, $($arg)*)))
    };
}

/// Bail out with a user error
#[macro_export]
macro_rules! bail_user {
    ($msg:expr) => {
        return Err($crate::user_error!($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::user_error!($fmt, $($arg)*))
    };
}

/// Ensure a condition is true or bail with user error
#[macro_export]
macro_rules! ensure_user {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            $crate::bail_user!($msg);
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            $crate::bail_user!($fmt, $($arg)*);
        }
    };
}