// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Context {
    None,
    Tag(String),
}

#[allow(unused_macros)]
macro_rules! trace {
    ($ctx:expr, $pat:expr $(,)?) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::trace!(concat!("{}: ", $pat), tag)
            }
            $crate::log::Context::None => {
                tracing::trace!($pat)
            }
        }
    };
    ($ctx:expr, $pat:expr, $($arg:tt)+) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::trace!(concat!("{}: ", $pat), tag, $($arg)+)
            }
            $crate::log::Context::None => {
                tracing::trace!($pat, $($arg)+)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! debug {
    ($ctx:expr, $pat:expr $(,)?) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::debug!(concat!("{}: ", $pat), tag)
            }
            $crate::log::Context::None => {
                tracing::debug!($pat)
            }
        }
    };
    ($ctx:expr, $pat:expr, $($arg:tt)+) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::debug!(concat!("{}: ", $pat), tag, $($arg)+)
            }
            $crate::log::Context::None => {
                tracing::debug!($pat, $($arg)+)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! info {
    ($ctx:expr, $pat:expr $(,)?) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::info!(concat!("{}: ", $pat), tag)
            }
            $crate::log::Context::None => {
                tracing::info!($pat)
            }
        }
    };
    ($ctx:expr, $pat:expr, $($arg:tt)+) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::info!(concat!("{}: ", $pat), tag, $($arg)+)
            }
            $crate::log::Context::None => {
                tracing::info!($pat, $($arg)+)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! warn {
    ($ctx:expr, $pat:expr $(,)?) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::warn!(concat!("{}: ", $pat), tag)
            }
            $crate::log::Context::None => {
                tracing::warn!($pat)
            }
        }
    };
    ($ctx:expr, $pat:expr, $($arg:tt)+) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::warn!(concat!("{}: ", $pat), tag, $($arg)+)
            }
            $crate::log::Context::None => {
                tracing::warn!($pat, $($arg)+)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! error {
    ($ctx:expr, $pat:expr $(,)?) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::error!(concat!("{}: ", $pat), tag)
            }
            $crate::log::Context::None => {
                tracing::error!($pat)
            }
        }
    };
    ($ctx:expr, $pat:expr, $($arg:tt)+) => {
        match &$ctx {
            $crate::log::Context::Tag(tag) => {
                tracing::error!(concat!("{}: ", $pat), tag, $($arg)+)
            }
            $crate::log::Context::None => {
                tracing::error!($pat, $($arg)+)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing::field::{Field, Visit};
    use tracing::span::{Attributes, Id, Record};
    use tracing::{Event, Metadata, Subscriber};

    #[derive(Clone, Default)]
    struct LogCapture(Arc<Mutex<Vec<String>>>);

    struct Visitor<'a>(&'a mut String);
    impl Visit for Visitor<'_> {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                self.0.push_str(&format!("{value:?}"));
            }
        }
    }

    impl Subscriber for LogCapture {
        fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, _span: &Attributes<'_>) -> Id {
            Id::from_u64(1)
        }
        fn record(&self, _span: &Id, _values: &Record<'_>) {}
        fn record_follows_from(&self, _span: &Id, _follows: &Id) {}
        fn event(&self, event: &Event<'_>) {
            let mut msg = String::new();
            let mut visitor = Visitor(&mut msg);
            event.record(&mut visitor);
            self.0.lock().unwrap().push(msg);
        }
        fn enter(&self, _span: &Id) {}
        fn exit(&self, _span: &Id) {}
    }

    #[test]
    fn tag_context_prefixes_tag() {
        let capture = LogCapture::default();
        let subscriber = capture.clone();
        tracing::subscriber::with_default(subscriber, || {
            let ctx = Context::Tag("test-tag".to_string());
            trace!(ctx, "trace message");
            debug!(ctx, "debug message: {}", 42);
            info!(ctx, "info message: {} and {}", "foo", "bar");
            warn!(ctx, "warn message");
            error!(ctx, "error message: {}", "err");
        });

        let logs = capture.0.lock().unwrap().clone();
        assert_eq!(
            logs,
            vec![
                "test-tag: trace message",
                "test-tag: debug message: 42",
                "test-tag: info message: foo and bar",
                "test-tag: warn message",
                "test-tag: error message: err",
            ]
        );
    }

    #[test]
    fn none_context_no_prefix() {
        let capture = LogCapture::default();
        let subscriber = capture.clone();
        tracing::subscriber::with_default(subscriber, || {
            let ctx = Context::None;
            trace!(ctx, "trace message");
            debug!(ctx, "debug message: {}", 42);
            info!(ctx, "info message: {} and {}", "foo", "bar");
            warn!(ctx, "warn message");
            error!(ctx, "error message: {}", "err");
        });

        let logs = capture.0.lock().unwrap().clone();
        assert_eq!(
            logs,
            vec![
                "trace message",
                "debug message: 42",
                "info message: foo and bar",
                "warn message",
                "error message: err",
            ]
        );
    }

    #[test]
    fn context_passed_by_ref() {
        let capture = LogCapture::default();
        let subscriber = capture.clone();
        tracing::subscriber::with_default(subscriber, || {
            let ctx = Context::Tag("ref-tag".to_string());
            debug!(&ctx, "by ref: {}", 1);
            debug!(ctx, "by val: {}", 2);
        });

        let logs = capture.0.lock().unwrap().clone();
        assert_eq!(logs, vec!["ref-tag: by ref: 1", "ref-tag: by val: 2"]);
    }

    #[test]
    fn trailing_commas() {
        let capture = LogCapture::default();
        let subscriber = capture.clone();
        tracing::subscriber::with_default(subscriber, || {
            let ctx = Context::Tag("comma-tag".to_string());
            debug!(ctx, "no args",);
            debug!(ctx, "one arg: {}", 1,);
            debug!(Context::None, "no args",);
            debug!(Context::None, "one arg: {}", 1,);
        });

        let logs = capture.0.lock().unwrap().clone();
        assert_eq!(
            logs,
            vec!["comma-tag: no args", "comma-tag: one arg: 1", "no args", "one arg: 1",]
        );
    }
}
