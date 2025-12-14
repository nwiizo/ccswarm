/// Command pattern macros for reducing boilerplate in CLI command implementations
///
/// These macros standardize command creation and execution patterns.
/// Define a command with automatic builder and execution
#[macro_export]
macro_rules! define_command {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $type:ty $(= $default:expr)?
            ),* $(,)?
        }
        execute($self:ident) $body:block
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $type,
            )*
        }

        impl $name {
            /// Create new command builder
            pub fn builder() -> paste::paste! { [<$name Builder>] } {
                paste::paste! { [<$name Builder>]::default() }
            }
        }

        #[async_trait]
        impl $crate::cli::Command for $name {
            async fn execute(&$self) -> Result<()> $body

            fn name(&self) -> &'static str {
                stringify!($name)
            }
        }

        paste::paste! {
            #[derive(Default)]
            pub struct [<$name Builder>] {
                $(
                    $field: Option<$type>,
                )*
            }

            impl [<$name Builder>] {
                $(
                    pub fn $field(mut self, $field: $type) -> Self {
                        self.$field = Some($field);
                        self
                    }
                )*

                pub fn build(self) -> Result<$name> {
                    Ok($name {
                        $(
                            $field: self.$field
                                $(.or(Some($default)))?
                                .ok_or_else(|| anyhow::anyhow!(concat!("Missing required field: ", stringify!($field))))?,
                        )*
                    })
                }
            }
        }
    };
}

/// Create a command registry with automatic dispatch
#[macro_export]
macro_rules! command_registry {
    (
        $registry:ident {
            $(
                $command:ident => $handler:expr
            ),* $(,)?
        }
    ) => {
        pub struct $registry {
            commands: HashMap<&'static str, Box<dyn Fn() -> Box<dyn $crate::cli::Command + Send + Sync> + Send + Sync>>,
        }

        impl $registry {
            pub fn new() -> Self {
                let mut commands = HashMap::new();

                $(
                    commands.insert(
                        stringify!($command),
                        Box::new(|| Box::new($handler) as Box<dyn $crate::cli::Command + Send + Sync>)
                            as Box<dyn Fn() -> Box<dyn $crate::cli::Command + Send + Sync> + Send + Sync>
                    );
                )*

                Self { commands }
            }

            pub fn get(&self, name: &str) -> Option<Box<dyn $crate::cli::Command + Send + Sync>> {
                self.commands.get(name).map(|factory| factory())
            }

            pub fn list_commands(&self) -> Vec<&'static str> {
                self.commands.keys().copied().collect()
            }
        }
    };
}

/// Define a subcommand group
#[macro_export]
macro_rules! subcommand_group {
    (
        $group:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident($command:ty)
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub enum $group {
            $(
                $(#[$variant_meta])*
                $variant($command),
            )*
        }

        #[async_trait]
        impl $crate::cli::Command for $group {
            async fn execute(&self) -> Result<()> {
                match self {
                    $(
                        Self::$variant(cmd) => cmd.execute().await,
                    )*
                }
            }

            fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant(_) => stringify!($variant),
                    )*
                }
            }
        }
    };
}

/// Create async command handler with error handling
#[macro_export]
macro_rules! async_command {
    ($name:ident($($param:ident: $type:ty),*) -> Result<$ret:ty> $body:block) => {
        pub async fn $name($($param: $type),*) -> Result<$ret> {
            let _span = tracing::info_span!(stringify!($name), $($param = ?$param),*).entered();

            let result = async move $body.await;

            match &result {
                Ok(_) => tracing::info!("Command {} completed successfully", stringify!($name)),
                Err(e) => tracing::error!("Command {} failed: {:#}", stringify!($name), e),
            }

            result
        }
    };
}

/// Define CLI argument parser
#[macro_export]
macro_rules! cli_args {
    (
        $name:ident {
            $(
                $(#[$arg_meta:meta])*
                $arg:ident: $type:ty = $parse:expr
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(
                $(#[$arg_meta])*
                pub $arg: $type,
            )*
        }

        impl $name {
            pub fn parse(args: &clap::ArgMatches) -> Result<Self> {
                Ok(Self {
                    $(
                        $arg: $parse(args)
                            .context(concat!("Failed to parse argument: ", stringify!($arg)))?,
                    )*
                })
            }
        }
    };
}

/// Create command with automatic progress tracking
#[macro_export]
macro_rules! command_with_progress {
    ($name:ident, $message:expr, $body:expr) => {{
        use $crate::cli::progress::{ProgressStyle, ProgressTracker};

        let progress = ProgressTracker::new($message, ProgressStyle::Spinner);
        ProgressTracker::start(progress.clone()).await;

        let result = $body;

        match &result {
            Ok(_) => {
                ProgressTracker::complete(progress, true, None).await;
            }
            Err(e) => {
                ProgressTracker::complete(progress, false, Some(format!("{:#}", e))).await;
            }
        }

        result
    }};
}

/// Define validation rules for commands
#[macro_export]
macro_rules! validate_command {
    ($command:expr, {
        $($field:ident: $validation:expr),* $(,)?
    }) => {{
        $(
            if !$validation(&$command.$field) {
                return Err(anyhow::anyhow!(
                    "Validation failed for field '{}': invalid value {:?}",
                    stringify!($field),
                    $command.$field
                ));
            }
        )*
        Ok(())
    }};
}
