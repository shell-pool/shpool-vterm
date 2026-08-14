// Copyright 2025 Google LLC
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

#[cfg(feature = "unstable-internal-test")]
macro_rules! test_pub {
    ($(#[$meta:meta])* struct $($tt:tt)+) => {
        $(#[$meta])*
        pub struct $($tt)+
    };
    ($(#[$meta:meta])* enum $($tt:tt)+) => {
        $(#[$meta])*
        pub enum $($tt)+
    };
    ($(#[$meta:meta])* trait $($tt:tt)+) => {
        $(#[$meta])*
        pub trait $($tt)+
    };
    ($(#[$meta:meta])* fn $($tt:tt)+) => {
        $(#[$meta])*
        pub fn $($tt)+
    };
}

#[cfg(not(feature = "unstable-internal-test"))]
macro_rules! test_pub {
    ($(#[$meta:meta])* struct $($tt:tt)+) => {
        $(#[$meta])*
        pub(crate) struct $($tt)+
    };
    ($(#[$meta:meta])* enum $($tt:tt)+) => {
        $(#[$meta])*
        pub(crate) enum $($tt)+
    };
    ($(#[$meta:meta])* trait $($tt:tt)+) => {
        $(#[$meta])*
        pub(crate) trait $($tt)+
    };
    ($(#[$meta:meta])* fn $($tt:tt)+) => {
        $(#[$meta])*
        pub(crate) fn $($tt)+
    };
}
