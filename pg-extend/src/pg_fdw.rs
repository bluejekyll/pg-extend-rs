//! A trait for implementing a foreign data wrapper.
//! Adapted and transalated from
//! https://github.com/slaught/dummy_fdw/blob/master/dummy_data.c
//! and
//! https://bitbucket.org/adunstan/rotfang-fdw/src/ca21c2a2e5fa6e1424b61bf0170adb3ab4ae68e7/src/rotfang_fdw.c?at=master&fileviewer=file-view-default
//! For use with `#[pg_foreignwrapper]` from pg-extend-attr


use crate::{pg_datum, pg_error, pg_sys, pg_type};
use std::boxed::Box;
use std::collections::HashMap;
use std::ffi::CStr;

// TODO: can we avoid this box?
/// The foreign data wrapper itself. The next() method of this object
/// is responsible for creating row objects to return data.
/// The object is only active for the lifetime of a query, so it
/// is not an appropriate place to put caching or long-running connections.
pub trait ForeignData: Iterator<Item = Box<ForeignRow>> {
    /// Called when a scan is initiated. Note that any heavy set up
    /// such as making connections or allocating memory should not
    /// happen in this step, but on the first call to next()
    fn begin(server_opts: OptionMap, table_opts: OptionMap) -> Self;
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
        let wrapper = Box::new(Self {
            state: T::begin(server_opts, table_opts)
        });

        (*node).fdw_state = Box::into_raw(wrapper) as *mut std::os::raw::c_void;
    }

    fn get_field(
        attr: &pg_sys::FormData_pg_attribute,
        row: &ForeignRow,
    ) -> Result<Option<pg_datum::PgDatum>, String> {
        let cname = unsafe { CStr::from_ptr(attr.attname.data.as_ptr()) };
        let name = cname
            .to_str()
            // TODO
            .map_err(|e| format!("{:#?}", e))?;
        // let typ = attr.atttypid;
        // TODO not fake
        let typ = pg_type::PgType::Text;
        // TODO get options
        let opts = HashMap::new();
        row.get_field(name, typ, opts).map_err(|e| e.into())
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
            // Get list of attributes
            #[cfg(feature = "postgres-11")]
            let (tupledesc, attrs) = {
                let tupledesc = (*(*node).ss.ss_currentRelation).rd_att;
                let attrs: &[pg_sys::Form_pg_attribute] = 
                    std::slice::from_raw_parts((*tupledesc).attrs.as_mut_ptr() as *const *mut _, (*tupledesc).natts as usize);
                
                (tupledesc, attrs)
            };

            #[cfg(not(feature = "postgres-11"))]
            let (mut tupledesc, attrs) = {
                let tupledesc = *(*(*node).ss.ss_currentRelation).rd_att;
                let attrs: &[pg_sys::Form_pg_attribute] =
                    std::slice::from_raw_parts(tupledesc.attrs, tupledesc.natts as usize);

                (tupledesc, attrs)
            };

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
                &mut tupledesc as *mut _,
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
            AddForeignUpdateTargets: None,
            PlanForeignModify: None,
            BeginForeignModify: None,

            ExecForeignInsert: None,
            ExecForeignUpdate: None,
            ExecForeignDelete: None,
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
            ImportForeignSchema: None,
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
