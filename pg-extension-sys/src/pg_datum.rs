use crate::pg_bool::Bool;
use crate::pg_sys::{self, Datum, Oid};

pub struct PgDatum(Option<Datum>);

impl PgDatum {
    pub fn from_raw<B: Into<bool>>(datum: Datum, is_null: B) -> Self {
        let datum = if is_null.into() { None } else { Some(datum) };
        PgDatum(datum)
    }

    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }

    pub fn into_datum(self) -> Datum {
        match self.0 {
            Some(datum) => datum,
            None => 0 as Datum,
        }
    }
}

pub trait TryFromPgDatum: Sized {
    fn try_from(datum: PgDatum) -> Result<Self, &'static str>;
}

impl TryFromPgDatum for i32 {
    fn try_from(datum: PgDatum) -> Result<Self, &'static str> {
        if let Some(datum) = datum.0 {
            Ok(datum as i32)
        } else {
            Err("datum was NULL")
        }
    }
}

impl From<i32> for PgDatum {
    fn from(value: i32) -> Self {
        PgDatum(Some(value as Datum))
    }
}

impl From<()> for PgDatum {
    fn from(value: ()) -> Self {
        PgDatum(None)
    }
}

/*
#define PG_GETARG_DATUM(n)   (fcinfo->arg[n])
#define PG_GETARG_INT32(n)   DatumGetInt32(PG_GETARG_DATUM(n))
#define PG_GETARG_UINT32(n)  DatumGetUInt32(PG_GETARG_DATUM(n))
#define PG_GETARG_UINT64(n)  DatumGetUInt64(PG_GETARG_DATUM(n))
#define PG_GETARG_INT16(n)   DatumGetInt16(PG_GETARG_DATUM(n))
#define PG_GETARG_UINT16(n)  DatumGetUInt16(PG_GETARG_DATUM(n))
#define PG_GETARG_CHAR(n)    DatumGetChar(PG_GETARG_DATUM(n))
#define PG_GETARG_BOOL(n)    DatumGetBool(PG_GETARG_DATUM(n))
#define PG_GETARG_OID(n)     DatumGetObjectId(PG_GETARG_DATUM(n))
#define PG_GETARG_POINTER(n) DatumGetPointer(PG_GETARG_DATUM(n))
#define PG_GETARG_CSTRING(n) DatumGetCString(PG_GETARG_DATUM(n))
#define PG_GETARG_NAME(n)    DatumGetName(PG_GETARG_DATUM(n))
/* these macros hide the pass-by-reference-ness of the datatype: */
#define PG_GETARG_FLOAT4(n)  DatumGetFloat4(PG_GETARG_DATUM(n))
#define PG_GETARG_FLOAT8(n)  DatumGetFloat8(PG_GETARG_DATUM(n))
#define PG_GETARG_INT64(n)   DatumGetInt64(PG_GETARG_DATUM(n))
/* use this if you want the raw, possibly-toasted input datum: */
#define PG_GETARG_RAW_VARLENA_P(n)  ((struct varlena *) PG_GETARG_POINTER(n))
/* use this if you want the input datum de-toasted: */
#define PG_GETARG_VARLENA_P(n) PG_DETOAST_DATUM(PG_GETARG_DATUM(n))
/* and this if you can handle 1-byte-header datums: */
#define PG_GETARG_VARLENA_PP(n) PG_DETOAST_DATUM_PACKED(PG_GETARG_DATUM(n))
/* DatumGetFoo macros for varlena types will typically look like this: */
#define DatumGetByteaPP(X)          ((bytea *) PG_DETOAST_DATUM_PACKED(X))
#define DatumGetTextPP(X)           ((text *) PG_DETOAST_DATUM_PACKED(X))
#define DatumGetBpCharPP(X)         ((BpChar *) PG_DETOAST_DATUM_PACKED(X))
#define DatumGetVarCharPP(X)        ((VarChar *) PG_DETOAST_DATUM_PACKED(X))
#define DatumGetHeapTupleHeader(X)  ((HeapTupleHeader) PG_DETOAST_DATUM(X))
/* And we also offer variants that return an OK-to-write copy */
#define DatumGetByteaPCopy(X)       ((bytea *) PG_DETOAST_DATUM_COPY(X))
#define DatumGetTextPCopy(X)        ((text *) PG_DETOAST_DATUM_COPY(X))
#define DatumGetBpCharPCopy(X)      ((BpChar *) PG_DETOAST_DATUM_COPY(X))
#define DatumGetVarCharPCopy(X)     ((VarChar *) PG_DETOAST_DATUM_COPY(X))
#define DatumGetHeapTupleHeaderCopy(X)  ((HeapTupleHeader) PG_DETOAST_DATUM_COPY(X))
/* Variants which return n bytes starting at pos. m */
#define DatumGetByteaPSlice(X,m,n)  ((bytea *) PG_DETOAST_DATUM_SLICE(X,m,n))
#define DatumGetTextPSlice(X,m,n)   ((text *) PG_DETOAST_DATUM_SLICE(X,m,n))
#define DatumGetBpCharPSlice(X,m,n) ((BpChar *) PG_DETOAST_DATUM_SLICE(X,m,n))
#define DatumGetVarCharPSlice(X,m,n) ((VarChar *) PG_DETOAST_DATUM_SLICE(X,m,n))
/* GETARG macros for varlena types will typically look like this: */
#define PG_GETARG_BYTEA_PP(n)       DatumGetByteaPP(PG_GETARG_DATUM(n))
#define PG_GETARG_TEXT_PP(n)        DatumGetTextPP(PG_GETARG_DATUM(n))
#define PG_GETARG_BPCHAR_PP(n)      DatumGetBpCharPP(PG_GETARG_DATUM(n))
#define PG_GETARG_VARCHAR_PP(n)     DatumGetVarCharPP(PG_GETARG_DATUM(n))
#define PG_GETARG_HEAPTUPLEHEADER(n)    DatumGetHeapTupleHeader(PG_GETARG_DATUM(n))
/* And we also offer variants that return an OK-to-write copy */
#define PG_GETARG_BYTEA_P_COPY(n)   DatumGetByteaPCopy(PG_GETARG_DATUM(n))
#define PG_GETARG_TEXT_P_COPY(n)    DatumGetTextPCopy(PG_GETARG_DATUM(n))
#define PG_GETARG_BPCHAR_P_COPY(n)  DatumGetBpCharPCopy(PG_GETARG_DATUM(n))
#define PG_GETARG_VARCHAR_P_COPY(n) DatumGetVarCharPCopy(PG_GETARG_DATUM(n))
#define PG_GETARG_HEAPTUPLEHEADER_COPY(n)   DatumGetHeapTupleHeaderCopy(PG_GETARG_DATUM(n))
/* And a b-byte slice from position a -also OK to write */
#define PG_GETARG_BYTEA_P_SLICE(n,a,b) DatumGetByteaPSlice(PG_GETARG_DATUM(n),a,b)
#define PG_GETARG_TEXT_P_SLICE(n,a,b)  DatumGetTextPSlice(PG_GETARG_DATUM(n),a,b)
#define PG_GETARG_BPCHAR_P_SLICE(n,a,b) DatumGetBpCharPSlice(PG_GETARG_DATUM(n),a,b)
#define PG_GETARG_VARCHAR_P_SLICE(n,a,b) DatumGetVarCharPSlice(PG_GETARG_DATUM(n),a,b)
/*
 * Obsolescent variants that guarantee INT alignment for the return value.
 * Few operations on these particular types need alignment, mainly operations
 * that cast the VARDATA pointer to a type like int16[].  Most code should use
 * the ...PP(X) counterpart.  Nonetheless, these appear frequently in code
 * predating the PostgreSQL 8.3 introduction of the ...PP(X) variants.
 */
#define DatumGetByteaP(X)           ((bytea *) PG_DETOAST_DATUM(X))
#define DatumGetTextP(X)            ((text *) PG_DETOAST_DATUM(X))
#define DatumGetBpCharP(X)          ((BpChar *) PG_DETOAST_DATUM(X))
#define DatumGetVarCharP(X)         ((VarChar *) PG_DETOAST_DATUM(X))
#define PG_GETARG_BYTEA_P(n)        DatumGetByteaP(PG_GETARG_DATUM(n))
#define PG_GETARG_TEXT_P(n)         DatumGetTextP(PG_GETARG_DATUM(n))
#define PG_GETARG_BPCHAR_P(n)       DatumGetBpCharP(PG_GETARG_DATUM(n))
#define PG_GETARG_VARCHAR_P(n)      DatumGetVarCharP(PG_GETARG_DATUM(n))
*/
