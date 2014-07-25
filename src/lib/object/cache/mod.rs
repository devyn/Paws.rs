//! Caches for common operations on Paws objects.

use object::{ObjectRef, WeakObjectRef};

use std::sync::Arc;
use std::collections::LruCache;

#[cfg(test)]
mod tests;

static SYM_LOOKUP_CACHE_SIZE: uint = 64;

/// Provides caching for various common operations on Paws objects.
pub struct Cache {
  sym_lookup_cache: LruCache<SymLookupCacheKey, SymLookupCacheEntry>,
  stats:            CacheStats
}

/// Provides performance-related information for a `Cache`.
pub struct CacheStats {
  /// The number of times `sym_lookup()` has failed to find a match in the cache
  /// since it was created.
  pub sym_lookup_misses: u64,

  /// The number of times `sym_lookup()` has successfully found a match in the
  /// cache since it was created.
  pub sym_lookup_hits:   u64
}

#[allow(raw_pointer_deriving)]
#[deriving(Hash, PartialEq, Eq)]
struct SymLookupCacheKey(ObjectRef, *const String);

struct SymLookupCacheEntry {
  container_version: uint,
  pair_version:      uint,
  pair:              WeakObjectRef,
  value:             WeakObjectRef
}

impl Cache {
  /// Construct a new cache.
  pub fn new() -> Cache {
    Cache {
      sym_lookup_cache: LruCache::new(SYM_LOOKUP_CACHE_SIZE),

      stats: CacheStats {
        sym_lookup_misses: 0,
        sym_lookup_hits:   0
      }
    }
  }

  /// Get information about cache performance.
  pub fn stats(&self) -> &CacheStats {
    &self.stats
  }

  /// Cache-optimized variant of `Members::lookup_pair()` specialized for
  /// lookups with a Symbol key only.
  pub fn sym_lookup(&mut self,
                    container: ObjectRef,
                    symbol:    Arc<String>)
                    -> Option<ObjectRef> {

    let key = SymLookupCacheKey(container, &*symbol as *const String);

    match self.sym_lookup_cache.get(&key) {
      Some(entry) => {
        // The lookup was cached. Let's check to see whether it's still valid.
        //
        // First, we need to ensure that neither the container nor the pair have
        // changed since we cached this entry.
        let SymLookupCacheKey(ref container, _) = key;

        if container.meta_version() == entry.container_version &&
           entry.pair.upgrade().map(|pair|
             pair.meta_version() == entry.pair_version) == Some(true) {

          self.stats.sym_lookup_hits += 1;

          debug!("sym_lookup  hit: ({} hits / {} misses)",
            self.stats.sym_lookup_hits, self.stats.sym_lookup_misses);

          // Return the associated value. The WeakObjectRef should always be
          // upgradeable unless something has gone horribly wrong, because the
          // metadata version of the pair has not changed.
          return Some(entry.value.upgrade()
            .expect("A valid SymLookupCacheEntry's value failed to upgrade()!"))
        }
      },
      None => ()
    };

    // The cache did not contain a valid entry for the key, so let's create
    // one by implementing a variant of the lookup_pair() algorithm.
    let     container_version: uint;
    let mut pair_version: Option<uint> = None;

    let mut result: Option<(ObjectRef, ObjectRef)> = None;

    self.stats.sym_lookup_misses += 1;

    debug!("sym_lookup miss: ({} hits / {} misses)",
      self.stats.sym_lookup_hits, self.stats.sym_lookup_misses);
    
    {
      let SymLookupCacheKey(ref container_ref, sym) = key;
      let container = container_ref.lock();
      let members   = &container.meta().members;

      // It's important that this happens *while* we have the lock.
      container_version = container_ref.meta_version();

      // Iterate through the members, looking for pair-shaped objects with
      // symbol keys that match the symbol key we're looking for.
      for maybe_relationship in members.iter().rev() {
        match maybe_relationship {
          &Some(ref relationship) => {
            let object  = relationship.to().lock();
            let members = &object.meta().members;

            // Pair objects look approximately like [, key, value].
            match (members.get(1), members.get(2)) {
              (Some(rel_key), Some(rel_value)) =>
                if sym_match(sym, rel_key.to()) {
                  // This is the matching pair.
                  //
                  // It's important that we check the version **while** we
                  // have the lock for this pair object.
                  pair_version = Some(relationship.to().meta_version());

                  result = Some((relationship.to().clone(),
                                 rel_value.to().clone()));

                  break;
                },
              _ => ()
            }
          },
          _ => ()
        }
      }
    }

    // Now check to see if we actually found anything, and if we did, update
    // the cache accordingly.
    match result {
      Some((pair, value)) => {
        let entry = SymLookupCacheEntry {
          container_version: container_version,
          pair_version:      pair_version.unwrap(),
          pair:              pair.downgrade(),
          value:             value.downgrade()
        };

        self.sym_lookup_cache.put(key, entry);

        // Return the value we found.
        return Some(value)
      },
      None => {
        // We didn't find anything, so let's make sure we remove the key from
        // the cache (if it existed) so we aren't doing this over and over.
        self.sym_lookup_cache.pop(&key);

        return None
      }
    }

    fn sym_match(sym: *const String, object: &ObjectRef) -> bool {
      match object.symbol_ref() {
        Some(other_sym) => sym == (&**other_sym as *const String),
        None            => false
      }
    }
  }
}
