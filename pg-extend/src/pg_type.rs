//! Postgres type definitions

/// See https://www.postgresql.org/docs/11/xfunc-c.html#XFUNC-C-TYPE-TABLE
///
/// TODO: it would be cool to share code with the sfackler/rust-postgres project
/// though, that is converting from NetworkByte order, and this is all NativeByte order?
#[derive(Clone, Copy)]
pub enum PgType {
    /// abstime 	AbsoluteTime 	utils/nabstime.h
    AbsoluteTime,
    /// bigint (int8) 	int64 	postgres.h
    BigInt,
    /// bigint (int8) 	int64 	postgres.h
    Int8,
    /// boolean 	bool 	postgres.h (maybe compiler built-in)
    Boolean,
    /// box 	BOX* 	utils/geo_decls.h
    GeoBox,
    /// bytea 	bytea* 	postgres.h
    ByteA,
    /// "char" 	char 	(compiler built-in)
    Char,
    /// character 	BpChar* 	postgres.h
    Character,
    /// cid 	CommandId 	postgres.h
    CommandId,
    /// date 	DateADT 	utils/date.h
    Date,
    /// smallint (int2) 	int16 	postgres.h
    SmallInt,
    /// smallint (int2) 	int16 	postgres.h
    Int2,
    /// int2vector 	int2vector* 	postgres.h
    Int2Vector,
    /// integer (int4) 	int32 	postgres.h
    Integer,
    /// integer (int4) 	int32 	postgres.h
    Int4,
    /// real (float4) 	float4* 	postgres.h
    Real,
    /// real (float4) 	float4* 	postgres.h
    Float4,
    /// double precision (float8) 	float8* 	postgres.h
    DoublePrecision,
    /// double precision (float8) 	float8* 	postgres.h
    Float8,
    /// interval 	Interval* 	datatype/timestamp.h
    Interval,
    /// lseg 	LSEG* 	utils/geo_decls.h
    Lseg,
    /// name 	Name 	postgres.h
    Name,
    /// oid 	Oid 	postgres.h
    Oid,
    /// oidvector 	oidvector* 	postgres.h
    OidVector,
    /// path 	PATH* 	utils/geo_decls.h
    Path,
    /// point 	POINT* 	utils/geo_decls.h
    Point,
    /// regproc 	regproc 	postgres.h
    RegProc,
    /// reltime 	RelativeTime 	utils/nabstime.h
    RelativeTime,
    /// text 	text* 	postgres.h
    Text,
    /// tid 	ItemPointer 	storage/itemptr.h
    ItemPointer,
    /// time 	TimeADT 	utils/date.h
    Time,
    /// time with time zone 	TimeTzADT 	utils/date.h
    TimeWithTimeZone,
    /// timestamp 	Timestamp* 	datatype/timestamp.h
    Timestamp,
    /// tinterval 	TimeInterval 	utils/nabstime.h
    TimeInterval,
    /// varchar 	VarChar* 	postgres.h
    VarChar,
    /// void
    Void,
    /// xid 	TransactionId 	postgres.h
    TransactionId,
}

impl PgType {
    /// Return the PgType of the parameter's type
    pub fn from_rust<T: PgTypeInfo>() -> PgType {
        T::pg_type()
    }

    /// Return the string representation of this type
    pub fn as_str(self) -> &'static str {
        match self {
            // abstime 	AbsoluteTime 	utils/nabstime.h
            PgType::AbsoluteTime => "abstime",
            // bigint (int8) 	int64 	postgres.h
            PgType::BigInt => "bigint",
            PgType::Int8 => "int8",
            // boolean 	bool 	postgres.h (maybe compiler built-in)
            PgType::Boolean => "boolean",
            // box 	BOX* 	utils/geo_decls.h
            PgType::GeoBox => "box",
            // bytea 	bytea* 	postgres.h
            PgType::ByteA => "bytea",
            // "char" 	char 	(compiler built-in)
            PgType::Char => "char",
            // character 	BpChar* 	postgres.h
            PgType::Character => "character",
            // cid 	CommandId 	postgres.h
            PgType::CommandId => "cid",
            // date 	DateADT 	utils/date.h
            PgType::Date => "date",
            // smallint (int2) 	int16 	postgres.h
            PgType::SmallInt => "smallint",
            PgType::Int2 => "int2",
            // int2vector 	int2vector* 	postgres.h
            PgType::Int2Vector => "int2vector",
            // integer (int4) 	int32 	postgres.h
            PgType::Integer => "integer",
            PgType::Int4 => "int4",
            // real (float4) 	float4* 	postgres.h
            PgType::Real => "real",
            PgType::Float4 => "float4",
            // double precision (float8) 	float8* 	postgres.h
            PgType::DoublePrecision => "double precision",
            PgType::Float8 => "float8",
            // interval 	Interval* 	datatype/timestamp.h
            PgType::Interval => "interval",
            // lseg 	LSEG* 	utils/geo_decls.h
            PgType::Lseg => "lseg",
            // name 	Name 	postgres.h
            PgType::Name => "name",
            // oid 	Oid 	postgres.h
            PgType::Oid => "oid",
            // oidvector 	oidvector* 	postgres.h
            PgType::OidVector => "oidvector",
            // path 	PATH* 	utils/geo_decls.h
            PgType::Path => "path",
            // point 	POINT* 	utils/geo_decls.h
            PgType::Point => "point",
            // regproc 	regproc 	postgres.h
            PgType::RegProc => "regproc",
            // reltime 	RelativeTime 	utils/nabstime.h
            PgType::RelativeTime => "reltime",
            // text 	text* 	postgres.h
            PgType::Text => "text",
            // tid 	ItemPointer 	storage/itemptr.h
            PgType::ItemPointer => "tid",
            // time 	TimeADT 	utils/date.h
            PgType::Time => "time",
            // time with time zone 	TimeTzADT 	utils/date.h
            PgType::TimeWithTimeZone => "time with time zone",
            // timestamp 	Timestamp* 	datatype/timestamp.h
            PgType::Timestamp => "timestamp",
            // tinterval 	TimeInterval 	utils/nabstime.h
            PgType::TimeInterval => "tinterval",
            // varchar 	VarChar* 	postgres.h
            PgType::VarChar => "varchar",
            // void
            PgType::Void => "void",
            // xid 	TransactionId 	postgres.h
            PgType::TransactionId => "xid",
        }
    }

    /// Return the String to be used for the RETURNS statement in SQL
    pub fn return_stmt(self) -> String {
        format!("RETURNS {}", self.as_str())
    }
}

/// Get the Postgres info for a type
pub trait PgTypeInfo {
    /// return the Postgres type
    fn pg_type() -> PgType;
    /// for distinguishing optional and non-optional arguments
    fn is_option() -> bool { false }
}

impl PgTypeInfo for i16 {
    fn pg_type() -> PgType {
        PgType::Int2
    }
}

impl PgTypeInfo for i32 {
    fn pg_type() -> PgType {
        PgType::Int4
    }
}

impl PgTypeInfo for i64 {
    fn pg_type() -> PgType {
        PgType::Int8
    }
}

impl PgTypeInfo for String {
    fn pg_type() -> PgType {
        PgType::Text
    }
}

impl PgTypeInfo for std::ffi::CString {
    fn pg_type() -> PgType {
        PgType::Text
    }
}

impl PgTypeInfo for () {
    fn pg_type() -> PgType {
        PgType::Void
    }
}

impl<T> PgTypeInfo for Option<T> where T: PgTypeInfo {
    fn pg_type() -> PgType {
        T::pg_type()
    }

    fn is_option() -> bool { true }
}
