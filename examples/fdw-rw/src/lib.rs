// Copyright 2018 Liz Frost <web@stillinbeta.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::pg_alloc::PgAllocator;
use pg_extend::pg_datum::TryFromPgDatum;
use pg_extend::pg_fdw::{ForeignData, ForeignRow, OptionMap, Tuple};
use pg_extend::{info, pg_datum, pg_magic, pg_type};
use pg_extern_attr::pg_foreignwrapper;

use std::collections::HashMap;
use std::sync::RwLock;

// Needs feature(staticmutex)
// use std::sync::{StaticMutex, MUTEX_INIT};
// static LOCK: StaticMutex = MUTEX_INIT;
static mut _CACHE: Option<RwLock<HashMap<String, String>>> = None;

fn get_cache() -> &'static RwLock<HashMap<String, String>> {
    // let _g = LOCK.lock().unwrap();
    unsafe {
        if _CACHE.is_none() {
            let rw = RwLock::new(HashMap::new());
            _CACHE = Some(rw)
        }
        &_CACHE.as_ref().unwrap()
    }
}

// This tells Postges this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

#[pg_foreignwrapper]
struct CacheFDW {
    inner: Vec<(String, String)>,
}

struct MyRow {
    key: String,
    value: String,
}

impl ForeignRow for MyRow {
    fn get_field(
        &self,
        name: &str,
        _typ: pg_type::PgType,
        _opts: OptionMap,
    ) -> Result<Option<pg_datum::PgDatum>, &str> {
        match name {
            "key" => Ok(Some(self.key.clone().into())),
            "value" => Ok(Some(self.value.clone().into())),
            _ => Err("unknown field"),
        }
    }
}

impl Iterator for CacheFDW {
    type Item = Box<dyn ForeignRow>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.pop() {
            None => None,
            Some((k, v)) => Some(Box::new(MyRow {
                key: k.to_string(),
                value: v.to_string(),
            })),
        }
    }
}

impl ForeignData for CacheFDW {
    fn begin(_sopts: OptionMap, _topts: OptionMap, _table_name: String) -> Self {
        let c = get_cache().read().unwrap();
        let vecs = c.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        CacheFDW { inner: vecs }
    }

    fn schema(
        _sopts: OptionMap,
        server_name: String,
        _rschema: String,
        lschema: String,
    ) -> Option<Vec<String>> {
        Some(vec![format!(
            "
CREATE FOREIGN TABLE {schema}.mytable (
  key text,
  value text) SERVER {server}
",
            server = server_name,
            schema = lschema
        )])
    }

    fn index_columns(_sopts: OptionMap, _topts: OptionMap, _tn: String) -> Option<Vec<String>> {
        Some(vec!["key".into()])
    }

    fn update(&self, new_row: &Tuple, indices: &Tuple) -> Option<Box<dyn ForeignRow>> {
        let mut c = get_cache().write().unwrap();
        let key = indices.get("key");
        let value = new_row.get("value");
        match (key, value) {
            (Some(key), Some(value)) => {
                // TODO: handle errors

                // TODO: switch to currect memory context
                let memory_context = PgAllocator::current_context();

                let key = String::try_from(&memory_context, (*key).clone()).unwrap();
                let value = String::try_from(&memory_context, (*value).clone()).unwrap();
                c.insert(key.clone(), value.clone());
                Some(Box::new(MyRow { key, value }))
            }
            _ => {
                info!("Missing key ({:?}) or value ({:?})", key, value);
                None
            }
        }
    }

    fn insert(&self, new_row: &Tuple) -> Option<Box<dyn ForeignRow>> {
        // Since we only use one field from each, these methods are equivalent
        self.update(new_row, new_row)
    }

    fn delete(&self, indices: &Tuple) -> Option<Box<dyn ForeignRow>> {
        // TODO: switch to correct memory context
        let memory_context = PgAllocator::current_context();

        let mut c = get_cache().write().unwrap();
        let key = indices.get("key");

        match key {
            Some(key) => {
                let key = String::try_from(&memory_context, (*key).clone()).unwrap();
                match c.remove(&key) {
                    Some(value) => Some(Box::new(MyRow { key, value })),
                    None => None,
                }
            }
            _ => {
                info!("Delete called without Key");
                None
            }
        }
    }
}
