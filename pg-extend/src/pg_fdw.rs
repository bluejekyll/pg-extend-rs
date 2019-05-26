//! A trait for implementing a foreign data wrapper.
//! Adapted and transalated from
//! https://github.com/slaught/dummy_fdw/blob/master/dummy_data.c
//! and
//! https://bitbucket.org/adunstan/rotfang-fdw/src/ca21c2a2e5fa6e1424b61bf0170adb3ab4ae68e7/src/rotfang_fdw.c?at=master&fileviewer=file-view-default
//! For use with `#[pg_foreignwrapper]` from pg-extend-attr

use crate::{pg_datum, pg_error, pg_sys, pg_type};
use std::boxed::Box;
use std::collections::HashMap;
use std::ffi::{CStr,CString};

/// A map from column names to data types. Tuple order is not currently
/// preserved, it may be in the future.
pub type Tuple = HashMap<String, pg_datum::PgDatum>;

// TODO: can we avoid this box?
/// The foreign data wrapper itself. The next() method of this object
/// is responsible for creating row objects to return data.
/// The object is only active for the lifetime of a query, so it
/// is not an appropriate place to put caching or long-running connections.
pub trait ForeignData: Iterator<Item = Box<ForeignRow>> {
    /// Called when a scan is initiated. Note that any heavy set up
    /// such as making connections or allocating memory should not
    /// happen in this step, but on the first call to next()
    fn begin(server_opts: OptionMap, table_opts: OptionMap, table_name: String) -> Self;

    /// If defined, these columns will always be present in the tuple. This can
    /// be useful for update and delete operations, which otherwise might be
    /// missing key fields.
    fn index_columns(
        _server_opts: OptionMap,
        _table_opts: OptionMap,
        _table_name: String,
    ) -> Option<Vec<String>> {
        None
    }

    /// Method for IMPORT FOREIGN SCHEMA. Use one element per SQL statement to be
    /// executed.
    /// remote_schema and local_schema are the names of the "schema" (a
    /// collection of tables) passed to IMPORT FOREIGN SCHEMA.
    /// server_name is the name of the table.
    /// Returned statements must be of the form
    /// `CREATE FOREIGN TABLE local_schema.<tablename> (<fields>) SERVER server`
    /// Remote schema can be used or ignored.
    /// At present all other options passed in are ignored, in the future this
    /// method might take options for which tables to import.
    fn schema(_server_opts: OptionMap, _server_name: String, _remote_schema: String, _local_schema: String) -> Option<Vec<String>> {
        None
    }

    /// Method for UPDATEs. Takes in a new_row (which is a mapping of column
    /// names to values). indices is the same, but will always include columns
    /// specified by index_columns. Do not assume columns present in indices
    /// were present in the UPDATE statement.
    /// Returns the updated row, or None if no update occured.
    fn update(&self, _new_row: &Tuple, _indices: &Tuple) -> Option<Box<ForeignRow>> {
        pg_error::log(
            pg_error::Level::Error,
            file!(),
            line!(),
            module_path!(),
            "Table does not support update",
        );
        None
    }

    /// Method for INSERTs. Takes in new_row (which is a mapping of column
    /// names to values). Returns the inserted row, or None if no insert
    /// occurred.
    fn insert(&self, _new_row: &Tuple) -> Option<Box<ForeignRow>> {
        pg_error::log(
            pg_error::Level::Error,
            file!(),
            line!(),
            module_path!(),
            "Table does not support insert",
        );
        None
    }

    /// Method for DELETEs. Takes in a indices is the same, which consists of columns
    /// specified by index_columns.
    /// Returns the deleted row, or None if no row was deleted.
    fn delete(&self, _indices: &Tuple) -> Option<Box<ForeignRow>> {
        pg_error::log(
            pg_error::Level::Error,
            file!(),
            line!(),
            module_path!(),
            "Table does not support delete",
        );
        None
    }
}

/// The options passed to a server, table, or options
/// i.e. CREATE SERVER myserver FOREIGN DATA WRAPPER postgres_fdw
/// OPTIONS (host 'foo', dbname 'foodb', port '5432');
pub type OptionMap = HashMap<String, String>;

/// This represents a row. Because columns can be queried in any order,
/// no expectations can be made about the order to return fields in a row in.
/// Instead, choose which data to return at runtime.
pub trait ForeignRow {
    /// given a column name, type, and options, produce a value.
    /// The type of PgDatum returned _should_ match the column's type
    /// but this is not enforced.
    /// Use None to return a null, do not return a PgDatum::Null
    fn get_field(
        &self,
        name: &str,
        typ: pg_type::PgType,
        opts: OptionMap,
    ) -> Result<Option<pg_datum::PgDatum>, &str>;
}

/// Contains all the methods for interacting with
/// Postgres at a low level. You should not interact with this directly,
/// instead use `#[pg_foreignwrapper]` from pg-extend-attr
pub struct ForeignWrapper<T: ForeignData> {
    state: T,
}

impl<T: ForeignData> ForeignWrapper<T> {
    /// set relation size estimates for a foreign table
    unsafe extern "C" fn get_foreign_rel_size(
        _root: *mut pg_sys::PlannerInfo,
        base_rel: *mut pg_sys::RelOptInfo,
        _foreign_table_id: pg_sys::Oid,
    ) {
        (*base_rel).rows = 0.0;
    }

    /// create access path for a scan on the foreign table
    unsafe extern "C" fn get_foreign_paths(
        root: *mut pg_sys::PlannerInfo,
        base_rel: *mut pg_sys::RelOptInfo,
        _foreign_table_id: pg_sys::Oid,
    ) {
        /*
         * Create a ForeignPath node and add it as only possible path.  We use the
         * fdw_private list of the path to carry the convert_selectively option;
         * it will be propagated into the fdw_private list of the Plan node.
         */
        pg_sys::add_path(
            base_rel,
            pg_sys::create_foreignscan_path(
                root,
                base_rel,
                std::ptr::null_mut(),
                (*base_rel).rows,
                // TODO real costs
                pg_sys::Cost::from(10),
                pg_sys::Cost::from(0),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) as *mut pg_sys::Path,
        );
    }

    /// create a ForeignScan plan node
    unsafe extern "C" fn get_foreign_plan(
        _root: *mut pg_sys::PlannerInfo,
        baserel: *mut pg_sys::RelOptInfo,
        _foreigntableid: pg_sys::Oid,
        _best_path: *mut pg_sys::ForeignPath,
        tlist: *mut pg_sys::List,
        scan_clauses: *mut pg_sys::List,
        outer_plan: *mut pg_sys::Plan,
    ) -> *mut pg_sys::ForeignScan {
        let scan_relid = (*baserel).relid;
        let scan_clauses = pg_sys::extract_actual_clauses(scan_clauses, pgbool!(false));
        pg_sys::make_foreignscan(
            tlist,
            scan_clauses,
            scan_relid,
            scan_clauses,
            std::ptr::null_mut(), // fdw_private
            std::ptr::null_mut(), // fdw_scan_tlist
            std::ptr::null_mut(), // fdw_recheck_quals
            outer_plan,
        )
    }

    /// called during executor startup. perform any initialization
    /// needed, but not start the actual scan.
    unsafe extern "C" fn begin_foreign_scan(
        node: *mut pg_sys::ForeignScanState,
        _eflags: std::os::raw::c_int,
    ) {
        // TODO real server options
        let server_opts = HashMap::new();
        // TODO real table options
        let table_opts = HashMap::new();

        let rel = *(*node).ss.ss_currentRelation;
        let name = Self::get_table_name(&rel);
        let wrapper = Box::new(Self {
            state: T::begin(server_opts, table_opts, name),
        });

        (*node).fdw_state = Box::into_raw(wrapper) as *mut std::os::raw::c_void;
    }

    fn name_to_string(attname: pg_sys::NameData) -> String {
        let cname = unsafe { CStr::from_ptr(attname.data.as_ptr()) };
        match cname.to_str() {
            Ok(s) => s.into(),
            Err(err) => {
                pg_error::log(
                    pg_error::Level::Error,
                    file!(),
                    line!(),
                    module_path!(),
                    format!("Unicode error {}", err)
                );
                String::new()
            }
        }
    }

    unsafe fn get_table_name(rel: &pg_sys::RelationData) -> String {
        let table = pg_sys::GetForeignTable(rel.rd_id);
        let raw_name = pg_sys::get_rel_name((*table).relid);

        let cname = std::ffi::CStr::from_ptr(raw_name);
        match cname.to_str() {
            Ok(name) => name.into(),
            Err(err) => {
                pg_error::log(
                    pg_error::Level::Error,
                    file!(),
                    line!(),
                    module_path!(),
                    format!("Unicode error {}", err)
                );
                String::new()
            }
        }
    }


    fn get_field(
        attr: &pg_sys::FormData_pg_attribute,
        row: &ForeignRow,
    ) -> Result<Option<pg_datum::PgDatum>, String> {
        let name = Self::name_to_string(attr.attname);
        // let typ = attr.atttypid;
        // TODO not fake
        let typ = pg_type::PgType::Text;
        // TODO get options
        let opts = HashMap::new();
        row.get_field(&name, typ, opts).map_err(|e| e.into())
    }

    fn tts_to_hashmap(slot: *mut pg_sys::TupleTableSlot, tupledesc: &pg_sys::tupleDesc) -> Tuple {
        let attrs = unsafe { Self::tupdesc_attrs(tupledesc) };

        // Make sure the slot is fully populated
        unsafe {
            pg_sys::slot_getallattrs(slot)
        }

        let data: &[pg_sys::Datum] =
            unsafe { std::slice::from_raw_parts((*slot).tts_values, (*slot).tts_nvalid as usize) };

        let isnull =
            unsafe { std::slice::from_raw_parts((*slot).tts_isnull, (*slot).tts_nvalid as usize) };

        let mut t = HashMap::new();

        for i in 0..(attrs.len().min(data.len())) {
            let name = Self::name_to_string(unsafe {(*attrs[i]).attname});
            let data = pg_datum::PgDatum::from_raw(data[i], isnull[i]);
            t.insert(name, data);
        }

        t
    }

    unsafe fn tupdesc_attrs(tupledesc: &pg_sys::tupleDesc) -> &[pg_sys::Form_pg_attribute] {
        #[cfg(feature = "postgres-11")]
        #[allow(clippy::cast_ptr_alignment)]
        let attrs = (*tupledesc).attrs.as_ptr() as *const _;
        #[cfg(not(feature = "postgres-11"))]
        let attrs = (*tupledesc).attrs;

        std::slice::from_raw_parts(attrs, (*tupledesc).natts as usize)
    }

    /// Retrieve next row from the result set, or clear tuple slot to indicate
    ///	EOF.
    /// Fetch one row from the foreign
    ///  (the node's ScanTupleSlot should be used for this purpose).
    ///  Return NULL if no more rows are available.
    unsafe extern "C" fn iterate_foreign_scan(
        node: *mut pg_sys::ForeignScanState,
    ) -> *mut pg_sys::TupleTableSlot {
        let mut wrapper = Box::from_raw((*node).fdw_state as *mut Self);
        let slot = (*node).ss.ss_ScanTupleSlot;

        // clear the slot
        let slot = pg_sys::ExecClearTuple(slot);

        let ret = if let Some(row) = (*wrapper).state.next() {
            let tupledesc = (*(*node).ss.ss_currentRelation).rd_att;
            let attrs = Self::tupdesc_attrs(&*tupledesc);

            // Datum array
            let mut data = vec![0 as pg_sys::Datum; attrs.len()];
            // Boolean array
            let mut isnull = vec![pgbool!(true); attrs.len()];
            for (i, pattr) in attrs.iter().enumerate() {
                // TODO: There must be a better way to do this?
                let result = Self::get_field(&(**pattr), &(*row));
                match result {
                    Err(err) => {
                        pg_error::log(pg_error::Level::Warning, file!(), line!(), "get_field", err);
                        continue;
                    }
                    Ok(None) => continue,
                    Ok(Some(var)) => {
                        data[i] = var.into_datum();
                        isnull[i] = pgbool!(false);
                    }
                };
            }

            #[cfg(feature = "postgres-11")]
            let tuple = pg_sys::heap_form_tuple(
                tupledesc as *mut _,
                data.as_mut_slice().as_mut_ptr(),
                isnull.as_mut_slice().as_mut_ptr(),
            );

            #[cfg(not(feature = "postgres-11"))]
            let tuple = pg_sys::heap_form_tuple(
                tupledesc as *mut _,
                data.as_mut_slice().as_mut_ptr(),
                isnull.as_mut_slice().as_mut_ptr(),
            );

            pg_sys::ExecStoreTuple(
                tuple,
                slot,
                pg_sys::InvalidBuffer as pg_sys::Buffer,
                pgbool!(false),
            )
        } else {
            std::ptr::null_mut()
        };

        (*node).fdw_state = Box::into_raw(wrapper) as *mut std::ffi::c_void;
        ret
    }

    /// Restart the scan from the beginning
    unsafe extern "C" fn rescan_foreign_scan(_node: *mut pg_sys::ForeignScanState) {}

    /// End the scan and release resources.
    unsafe extern "C" fn end_foreign_scan(_node: *mut pg_sys::ForeignScanState) {}

    unsafe extern "C" fn add_foreign_update_targets(
        parsetree: *mut pg_sys::Query,
        _target_rte: *mut pg_sys::RangeTblEntry,
        target_relation: pg_sys::Relation
    ) {
        // TODO real server options
        let server_opts = HashMap::new();
        // TODO real table options
        let table_opts = HashMap::new();

        let table_name = Self::get_table_name(&*target_relation);

        if let Some(keys) = T::index_columns(
            server_opts,
            table_opts,
            table_name
        ) {

            // Build a map of column names to attributes and column index
            let attrs: HashMap<String, (&pg_sys::Form_pg_attribute, usize)> =
                Self::tupdesc_attrs(&*(*target_relation).rd_att)
                .iter()
                .enumerate()
                .map(|(idx, rel)| (Self::name_to_string((**rel).attname), (rel, idx)))
                .collect();

            for key in keys {
                // find the matching column
                let (attr, idx) = match attrs.get(&key) {
                    Some((attr, idx)) => (*(*attr), idx),
                    None => {
                        pg_error::log(
                            pg_error::Level::Error,
                            file!(),
                            line!(),
                            module_path!(),
                            format!("Table has no such key {}", key)
                        );
                        continue
                    }
                };

                let var = pg_sys::makeVar(
                    (*parsetree).resultRelation as u32,
                    *idx as i16 + 1, // points to the position in the tuple, 1-indexed
                    (*attr).atttypid,
                    (*attr).atttypmod,
                    0 as pg_sys::Oid, // InvalidOid
                    0
                );

                // TODO: error handling

                let ckey = std::ffi::CString::new(key).unwrap();
                let list = (*parsetree).targetList;
                let list_size = if list.is_null() {
                    0
                } else {
                    (*list).length
                };

                let tle = pg_sys::makeTargetEntry(
                    var as *mut pg_sys::Expr,
                    (list_size + 1) as i16,
                    pg_sys::pstrdup(ckey.as_ptr()),
                    pgbool!(true)
                );

                (*parsetree).targetList = pg_sys::lappend((*parsetree).targetList, tle as *mut std::ffi::c_void)
            }
        }
    }

    unsafe extern "C" fn begin_foreign_modify(
        _mstate: *mut pg_sys::ModifyTableState,
        rinfo: *mut pg_sys::ResultRelInfo,
        _fdw_private: *mut pg_sys::List,
        _subplan_index: i32,
        _eflags: i32,
    ) {
        // TODO real server options
        let server_opts = HashMap::new();
        // TODO real table options
        let table_opts = HashMap::new();

        let rel = *(*rinfo).ri_RelationDesc;
        let name = Self::get_table_name(&rel);
        let wrapper = Box::new(Self {
            state: T::begin(server_opts, table_opts, name),
        });

        (*rinfo).ri_FdwState = Box::into_raw(wrapper) as *mut std::ffi::c_void;
    }

    unsafe extern "C" fn exec_foreign_update(
        _estate: *mut pg_sys::EState,
        rinfo: *mut pg_sys::ResultRelInfo,
        slot: *mut pg_sys::TupleTableSlot,
        plan_slot: *mut pg_sys::TupleTableSlot,
    ) -> *mut pg_sys::TupleTableSlot {
        let wrapper = Box::from_raw((*rinfo).ri_FdwState as *mut Self);

        let fields = Self::tts_to_hashmap(slot, &*(*slot).tts_tupleDescriptor);
        let fields_with_index = Self::tts_to_hashmap(plan_slot, &*(*plan_slot).tts_tupleDescriptor);
        let result = (*wrapper).state.update(&fields, &fields_with_index);

        if result.is_none() {
            std::ptr::null_mut()
        } else {
            // TODO: actually use result
            slot
        }
    }

    unsafe extern "C" fn exec_foreign_delete(
        _estate: *mut pg_sys::EState,
        rinfo: *mut pg_sys::ResultRelInfo,
        slot: *mut pg_sys::TupleTableSlot,
        plan_slot: *mut pg_sys::TupleTableSlot,
    ) -> *mut pg_sys::TupleTableSlot {
        let wrapper = Box::from_raw((*rinfo).ri_FdwState as *mut Self);

        let fields_with_index = Self::tts_to_hashmap(plan_slot, &*(*plan_slot).tts_tupleDescriptor);

        let result = (*wrapper).state.delete(&fields_with_index);

        // TODO: Proper destructor for this
        (*rinfo).ri_FdwState = Box::into_raw(wrapper) as *mut std::ffi::c_void;

        if result.is_none() {
            std::ptr::null_mut()
        } else {
            // TODO: actually use result
            slot
        }
    }

    unsafe extern "C" fn exec_foreign_insert(
        _estate: *mut pg_sys::EState,
        rinfo: *mut pg_sys::ResultRelInfo,
        slot: *mut pg_sys::TupleTableSlot,
        _plan_slot: *mut pg_sys::TupleTableSlot,
    ) -> *mut pg_sys::TupleTableSlot {
        let wrapper = Box::from_raw((*rinfo).ri_FdwState as *mut Self);

        let tupledesc = (*(*rinfo).ri_RelationDesc).rd_att;
        let fields = Self::tts_to_hashmap(slot, &*tupledesc);

        let result = (*wrapper).state.insert(&fields);

        // TODO: Proper destructor for this
        (*rinfo).ri_FdwState = Box::into_raw(wrapper) as *mut std::ffi::c_void;

        if result.is_none() {
            std::ptr::null_mut()
        } else {
            // TODO: actually use result
            slot
        }
    }

    unsafe extern "C" fn import_foreign_schema(
        stmt: *mut pg_sys::ImportForeignSchemaStmt,
        _server_oid: pg_sys::Oid
    ) -> *mut pg_sys::List {
        // TODO real server opts
        let server_opts = HashMap::new();

        let server_name_cstr = CStr::from_ptr((*stmt).server_name);
        let remote_schema_cstr = CStr::from_ptr((*stmt).remote_schema);
        let local_schema_cstr = CStr::from_ptr((*stmt).local_schema);

        // TODO: handle unicode errors here
        let server_name = server_name_cstr.to_string_lossy().to_string();
        let remote_schema = remote_schema_cstr.to_string_lossy().to_string();
        let local_schema = local_schema_cstr.to_string_lossy().to_string();

        let stmts = match T::schema(server_opts, server_name, remote_schema, local_schema) {
            Some(s) => s,
            None => return std::ptr::null_mut(),
        };

        // Concat all the statements together
        let mut list = std::ptr::null_mut() as *mut pg_sys::List;

        for stmt in stmts {
            let cstmt = CString::new(stmt).unwrap();

            let dup = pg_sys::pstrdup(cstmt.as_ptr()) as *mut std::ffi::c_void;
            list = pg_sys::lappend(list, dup);
        }


        list
    }

    /// Turn this into an actual foreign data wrapper object.
    /// Postgres creates fdws by having a function return a special
    /// fdw_routine object, which is what this datum is.
    pub fn into_datum() -> pg_sys::Datum {
        let node = Box::new(pg_sys::FdwRoutine {
            type_: pg_sys::NodeTag_T_FdwRoutine,
            GetForeignRelSize: Some(Self::get_foreign_rel_size),
            GetForeignPaths: Some(Self::get_foreign_paths),
            GetForeignPlan: Some(Self::get_foreign_plan),
            BeginForeignScan: Some(Self::begin_foreign_scan),
            IterateForeignScan: Some(Self::iterate_foreign_scan),
            ReScanForeignScan: Some(Self::rescan_foreign_scan),
            EndForeignScan: Some(Self::end_foreign_scan),

            #[cfg(feature = "postgres-11")]
            BeginForeignInsert: None,
            #[cfg(feature = "postgres-11")]
            EndForeignInsert: None,
            #[cfg(feature = "postgres-11")]
            ReparameterizeForeignPathByChild: None,

            #[cfg(any(feature = "postgres-10", feature = "postgres-11"))]
            ShutdownForeignScan: None,
            #[cfg(any(feature = "postgres-10", feature = "postgres-11"))]
            ReInitializeDSMForeignScan: None,

            GetForeignJoinPaths: None,
            GetForeignUpperPaths: None,
            AddForeignUpdateTargets: Some(Self::add_foreign_update_targets),
            PlanForeignModify: None,
            BeginForeignModify: Some(Self::begin_foreign_modify),

            ExecForeignInsert: Some(Self::exec_foreign_insert),
            ExecForeignUpdate: Some(Self::exec_foreign_update),
            ExecForeignDelete: Some(Self::exec_foreign_delete),
            EndForeignModify: None,

            IsForeignRelUpdatable: None,
            PlanDirectModify: None,
            BeginDirectModify: None,
            IterateDirectModify: None,
            EndDirectModify: None,
            GetForeignRowMarkType: None,
            RefetchForeignRow: None,
            RecheckForeignScan: None,

            ExplainForeignScan: None,
            ExplainForeignModify: None,
            ExplainDirectModify: None,
            AnalyzeForeignTable: None,
            ImportForeignSchema: Some(Self::import_foreign_schema),
            IsForeignScanParallelSafe: None,

            EstimateDSMForeignScan: None,
            InitializeDSMForeignScan: None,

            InitializeWorkerForeignScan: None,
        });
        // TODO: this isn't quite right, it will never be from_raw loaded
        // so it won't be cleaned properly
        Box::into_raw(node) as pg_sys::Datum
    }
}
