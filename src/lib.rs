// Copyright 2020 CoD Technologies Corp.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Select over a list of futures.
//!
//! ## Usage
//!
//! ```
//! use async_select_all::SelectAll;
//! use futures::executor::block_on;
//!
//! async fn inc(i: i32) -> i32 {
//!     i + 1
//! }
//!
//! fn main() {
//!     let futures = vec![inc(10), inc(5)];
//!     let mut select_all = SelectAll::from(futures);
//!     let vec = block_on(async {
//!         let mut vec = Vec::with_capacity(select_all.len());
//!         while !select_all.is_empty() {
//!             let val = select_all.select().await;
//!             vec.push(val)
//!         }
//!         vec.sort();
//!         vec
//!     });
//!     assert_eq!(vec, vec![6, 11]);
//! }
//! ```

use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    struct SelectFuture<'a, F: Future> {
        #[pin] futures: &'a mut Vec<F> ,
    }
}

impl<'a, F: Future> SelectFuture<'a, F> {
    #[inline]
    fn new(futures: &'a mut Vec<F>) -> Self {
        Self { futures }
    }
}

impl<'a, F: Future> Future for SelectFuture<'a, F> {
    type Output = F::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut ready = None;

        let mut this = self.project();
        for (i, f) in this.futures.iter_mut().enumerate() {
            let p = unsafe { Pin::new_unchecked(f) };
            if let Poll::Ready(output) = p.poll(cx) {
                ready = Some((i, Poll::Ready(output)));
                break;
            }
        }

        if let Some((id, r)) = ready {
            this.futures.swap_remove(id);
            return r;
        }

        Poll::Pending
    }
}

/// An unbounded set of futures.
pub struct SelectAll<F: Future> {
    futures: Vec<F>,
}

impl<I> From<I> for SelectAll<I::Item>
where
    I: IntoIterator,
    I::Item: Future,
{
    #[inline]
    fn from(iter: I) -> Self {
        Self {
            futures: iter.into_iter().collect(),
        }
    }
}

impl<F: Future> SelectAll<F> {
    /// Constructs a new, empty `SelectAll`.
    /// The returned `SelectAll` does not contain any futures.
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        SelectAll {
            futures: Vec::new(),
        }
    }

    /// Returns the number of futures contained in the set.
    /// This represents the total number of in-flight futures.
    #[inline]
    pub fn len(&self) -> usize {
        self.futures.len()
    }

    /// Returns true if the set contains no futures.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Push a future into the set.
    /// This function submits the given future to the set for managing.
    /// This function will not call poll on the submitted future.
    /// The caller must ensure that `SelectAll::select` is called in order to receive task notifications.
    #[inline]
    pub fn push(&mut self, future: F) {
        self.futures.push(future);
    }

    /// Select over a list of futures.
    ///
    /// Upon completion the item resolved will be returned.
    ///
    /// There are no guarantees provided on the order of the list with the remaining futures.
    /// They might be swapped around, reversed, or completely random.
    ///
    /// # Panics
    /// This function will panic if the `SelectAll` specified contains no items.
    #[inline]
    pub async fn select(&mut self) -> F::Output {
        assert!(!self.futures.is_empty());
        SelectFuture::new(&mut self.futures).await
    }
}
