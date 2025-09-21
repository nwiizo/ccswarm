/// Enhanced error handling macros

/// Log and continue on error
#[macro_export]
macro_rules! log_error {
    ($result:expr) => {
        if let Err(e) = $result {
            eprintln!("Error: {}", e);
        }
    };
    ($result:expr, $msg:expr) => {
        if let Err(e) = $result {
            eprintln!("{}: {}", $msg, e);
        }
    };
}

/// Try operation with fallback value
#[macro_export]
macro_rules! try_or_default {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => Default::default(),
        }
    };
    ($expr:expr, $default:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => $default,
        }
    };
}

/// Chain multiple fallible operations
#[macro_export]
macro_rules! chain_result {
    ($first:expr $(, $rest:expr)*) => {
        $first $(.and_then(|_| $rest))*
    };
}

/// Retry operation on error
#[macro_export]
macro_rules! retry_on_error {
    ($expr:expr, $times:expr) => {{
        let mut result = $expr;
        for _ in 1..$times {
            if result.is_ok() {
                break;
            }
            result = $expr;
        }
        result
    }};
}

/// Create workspace path with consistent path building
#[macro_export]
macro_rules! workspace_path {
    ($base:expr, $($segment:expr),+) => {
        $crate::utils::FsUtils::build_path($base, &[$($segment),+])
    };
}

/// Ensure directory exists with context
#[macro_export]
macro_rules! ensure_dir {
    ($path:expr, $context:expr) => {
        $crate::utils::FsUtils::ensure_dir_exists($path, $context).await
    };
}

/// Create test task with default values
#[macro_export]
macro_rules! test_task {
    ($id:expr, $desc:expr) => {
        $crate::utils::TestSetup::create_test_task($id, $desc, $crate::agent::Priority::Medium)
    };
    ($id:expr, $desc:expr, $priority:expr) => {
        crate::utils::TestSetup::create_test_task($id, $desc, $priority)
    };
}

/// Wait for test processing with standard duration
#[macro_export]
macro_rules! wait_for_test {
    () => {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await
    };
    ($millis:expr) => {
        tokio::time::sleep(tokio::time::Duration::from_millis($millis)).await
    };
}

/// Unwrap with test context
#[macro_export]
macro_rules! test_unwrap {
    ($expr:expr, $msg:expr) => {
        $expr.unwrap_or_else(|e| panic!("{}: {:#}", $msg, e))
    };
}

/// Execute async filesystem operation with standard error handling
#[macro_export]
macro_rules! fs_op {
    (save_json: $data:expr, $path:expr, $name:expr) => {
        $crate::utils::FsUtils::save_json($data, $path, $name).await
    };
    (load_json: $path:expr, $name:expr) => {
        $crate::utils::FsUtils::load_json($path, $name).await
    };
    (write_file: $path:expr, $content:expr, $name:expr) => {
        crate::utils::FsUtils::write_file($path, $content, $name).await
    };
    (remove_dir: $path:expr, $name:expr) => {
        crate::utils::FsUtils::remove_dir_all($path, $name).await
    };
}

#[cfg(test)]
mod tests {
    
    use anyhow::Result;

    #[test]
    fn test_try_or_default() {
        let result: Result<i32> = Ok(42);
        assert_eq!(try_or_default!(result), 42);

        let error_result: Result<i32> = Err(anyhow::anyhow!("error"));
        assert_eq!(try_or_default!(error_result), 0);

        assert_eq!(try_or_default!(error_result, 99), 99);
    }

    #[test]
    fn test_chain_result() {
        let result = chain_result!(
            Ok::<_, anyhow::Error>(1),
            Ok::<_, anyhow::Error>(2),
            Ok::<_, anyhow::Error>(3)
        );
        assert!(result.is_ok());

        let error_result = chain_result!(
            Ok::<_, anyhow::Error>(1),
            Err::<i32, _>(anyhow::anyhow!("error")),
            Ok::<_, anyhow::Error>(3)
        );
        assert!(error_result.is_err());
    }

    #[test]
    fn test_retry_on_error() {
        let mut counter = 0;
        let result = retry_on_error!({
            counter += 1;
            if counter < 3 {
                Err::<i32, _>(anyhow::anyhow!("error"))
            } else {
                Ok(42)
            }
        }, 5);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter, 3);
    }
}