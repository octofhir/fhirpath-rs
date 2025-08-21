// Copyright 2024 OctoFHIR Team
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

//! Priority queue implementation for background schema loading

use parking_lot::Mutex;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Notify;

/// Priority levels for schema loading requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    Essential = 0,  // Must load immediately
    Common = 1,     // Load early in background
    Requested = 2,  // Load when requested by user
    Predictive = 3, // Load based on usage patterns
}

/// Source of a schema loading request
#[derive(Debug, Clone)]
pub enum LoadRequester {
    Initialization,
    UserRequest(String),
    PredictiveSystem,
    AccessPattern,
}

/// A request to load a schema in the background
#[derive(Debug)]
pub struct SchemaLoadRequest {
    pub type_name: String,
    pub priority: LoadPriority,
    pub requested_at: Instant,
    pub requester: LoadRequester,
}

/// Priority queue item with ordering
#[derive(Debug)]
struct PriorityItem<T> {
    priority: LoadPriority,
    timestamp: Instant,
    item: T,
}

impl<T> PartialEq for PriorityItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl<T> Eq for PriorityItem<T> {}

impl<T> PartialOrd for PriorityItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for PriorityItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Primary sort by priority (lower number = higher priority)
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => {
                // Secondary sort by timestamp (older first)
                self.timestamp.cmp(&other.timestamp)
            }
            other => other,
        }
    }
}

/// Thread-safe priority queue for schema loading requests
#[derive(Debug)]
pub struct PriorityQueue<T> {
    heap: Arc<Mutex<BinaryHeap<Reverse<PriorityItem<T>>>>>,
    notify: Arc<Notify>,
}

impl<T> PriorityQueue<T> {
    /// Create a new priority queue
    pub fn new() -> Self {
        Self {
            heap: Arc::new(Mutex::new(BinaryHeap::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Add item to queue with priority
    pub fn push(&self, item: T, priority: LoadPriority) {
        let priority_item = PriorityItem {
            priority,
            timestamp: Instant::now(),
            item,
        };

        self.heap.lock().push(Reverse(priority_item));
        self.notify.notify_one();
    }

    /// Get highest priority item (blocks until available)
    pub async fn pop(&self) -> Option<T> {
        loop {
            // Try to get item without waiting
            {
                let mut heap = self.heap.lock();
                if let Some(Reverse(priority_item)) = heap.pop() {
                    return Some(priority_item.item);
                }
            }

            // Wait for notification of new items
            self.notify.notified().await;
        }
    }

    /// Try to get an item without blocking
    pub fn try_pop(&self) -> Option<T> {
        let mut heap = self.heap.lock();
        heap.pop().map(|Reverse(priority_item)| priority_item.item)
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.heap.lock().is_empty()
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.heap.lock().len()
    }

    /// Clear all items from the queue
    pub fn clear(&self) {
        self.heap.lock().clear();
    }

    /// Get items with a specific priority (for testing/debugging)
    pub fn items_with_priority(&self, priority: LoadPriority) -> usize {
        self.heap
            .lock()
            .iter()
            .filter(|Reverse(item)| item.priority == priority)
            .count()
    }
}

impl<T> Default for PriorityQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for PriorityQueue<T> {
    fn clone(&self) -> Self {
        Self {
            heap: self.heap.clone(),
            notify: self.notify.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_priority_ordering() {
        let queue = PriorityQueue::new();

        // Add items in different priority order
        queue.push("low".to_string(), LoadPriority::Predictive);
        queue.push("high".to_string(), LoadPriority::Essential);
        queue.push("medium".to_string(), LoadPriority::Requested);
        queue.push("high2".to_string(), LoadPriority::Common);

        // Should come out in priority order
        assert_eq!(queue.pop().await.unwrap(), "high");
        assert_eq!(queue.pop().await.unwrap(), "high2");
        assert_eq!(queue.pop().await.unwrap(), "medium");
        assert_eq!(queue.pop().await.unwrap(), "low");
    }

    #[tokio::test]
    async fn test_timestamp_ordering() {
        let queue = PriorityQueue::new();

        // Add items with same priority
        queue.push("first".to_string(), LoadPriority::Common);
        tokio::time::sleep(Duration::from_millis(10)).await; // Ensure different timestamps
        queue.push("second".to_string(), LoadPriority::Common);

        // Should come out in FIFO order for same priority
        assert_eq!(queue.pop().await.unwrap(), "first");
        assert_eq!(queue.pop().await.unwrap(), "second");
    }

    #[test]
    fn test_try_pop() {
        let queue = PriorityQueue::new();

        // Empty queue
        assert!(queue.try_pop().is_none());

        // Add item
        queue.push("test".to_string(), LoadPriority::Essential);
        assert_eq!(queue.try_pop().unwrap(), "test");

        // Empty again
        assert!(queue.try_pop().is_none());
    }

    #[test]
    fn test_queue_operations() {
        let queue = PriorityQueue::new();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.push("item1".to_string(), LoadPriority::Essential);
        queue.push("item2".to_string(), LoadPriority::Common);

        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 2);

        assert_eq!(queue.items_with_priority(LoadPriority::Essential), 1);
        assert_eq!(queue.items_with_priority(LoadPriority::Common), 1);
        assert_eq!(queue.items_with_priority(LoadPriority::Predictive), 0);

        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let queue = Arc::new(PriorityQueue::new());
        let queue_clone = queue.clone();

        // Producer task
        let producer = tokio::spawn(async move {
            for i in 0..10 {
                queue_clone.push(format!("item{i}"), LoadPriority::Common);
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        // Consumer task
        let consumer = tokio::spawn(async move {
            let mut count = 0;
            while count < 10 {
                if let Some(item) = queue.try_pop() {
                    assert!(item.starts_with("item"));
                    count += 1;
                } else {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
        });

        // Wait for both tasks to complete
        let (prod_result, cons_result) = tokio::join!(producer, consumer);
        prod_result.unwrap();
        cons_result.unwrap();
    }
}
