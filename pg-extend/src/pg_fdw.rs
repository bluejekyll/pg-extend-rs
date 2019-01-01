use crate::{pg_bool, pg_sys, pg_datum};
use std::marker::PhantomData;
use std::boxed::Box;

/// Adapted and transalated from https://github.com/slaught/dummy_fdw/blob/master/dummy_data.c
/// A trait for implementing a foreign data wrapper

pub trait ForeignData: IntoIterator<Item=Vec<pg_datum::PgDatum>> {
    fn new() -> Self;
}

pub struct ForeignWrapper<T: ForeignData>{
    wraps: PhantomData<T>
}

impl <T: ForeignData> ForeignWrapper<T> {
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
                10 as pg_sys::Cost,
                0 as pg_sys::Cost,
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
        let scan_clauses =
            pg_sys::extract_actual_clauses(scan_clauses, pg_bool::Bool::from(false).into());
        return pg_sys::make_foreignscan(
            tlist,
            scan_clauses,
            scan_relid,
            scan_clauses,
            std::ptr::null_mut(), // fdw_private
            std::ptr::null_mut(), // fdw_scan_tlist
            std::ptr::null_mut(), // fdw_recheck_quals
            outer_plan,
        );
    }

    /// called during executor startup. perform any initialization
    /// needed, but not start the actual scan.
    unsafe extern "C" fn begin_foreign_scan(
        node: *mut pg_sys::ForeignScanState,
        _eflags: std::os::raw::c_int,
    ) {
        let state = Box::new(T::new());
        (*node).fdw_state = Box::into_raw(state) as *mut std::os::raw::c_void;
    }

    /// Retrieve next row from the result set, or clear tuple slot to indicate
    ///	EOF.
    /// Fetch one row from the foreign
    ///  (the node's ScanTupleSlot should be used for this purpose).
    ///  Return NULL if no more rows are available.
    unsafe extern "C" fn iterate_foreign_scan(
        _node: *mut pg_sys::ForeignScanState,
    ) -> *mut pg_sys::TupleTableSlot {
        std::ptr::null_mut()
    }

    /// Restart the scan from the beginning
    unsafe extern "C" fn rescan_foreign_scan(_node: *mut pg_sys::ForeignScanState) {}

    /// End the scan and release resources.
    unsafe extern "C" fn end_foreign_scan(_node: *mut pg_sys::ForeignScanState) {}

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
        Box::into_raw(node) as pg_sys::Datum
    }
}
