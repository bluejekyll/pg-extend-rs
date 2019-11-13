// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#include "postgres.h"
#include "postgres_ext.h"
#include "access/relscan.h"
#include "access/sysattr.h"
#include "catalog/pg_type.h"
#include "executor/spi.h"
#include "foreign/fdwapi.h"
#include "foreign/foreign.h"
#include "lib/stringinfo.h"
#include "nodes/makefuncs.h"
#include "nodes/pg_list.h"
#include "nodes/memnodes.h"
#include "optimizer/pathnode.h"
#include "optimizer/planmain.h"
#include "optimizer/restrictinfo.h"

// Utils includes.
#include "utils/builtins.h"
#include "utils/json.h"
#include "utils/lsyscache.h"
#include "utils/lsyscache.h"
#include "utils/memutils.h"
#include "utils/palloc.h"
#include "utils/pg_lsn.h"
#include "utils/rel.h"

// Replication related includes.
#include "replication/basebackup.h"
#include "replication/logical.h"
#include "replication/logicallauncher.h"
#include "replication/logicalrelation.h"
#include "replication/message.h"
#include "replication/output_plugin.h"
#include "replication/reorderbuffer.h"
#include "replication/snapbuild.h"
#include "replication/walreceiver.h"
#include "replication/walsender_private.h"
#include "replication/decode.h"
#include "replication/logicalfuncs.h"
#include "replication/logicalproto.h"
#include "replication/logicalworker.h"
#include "replication/origin.h"
#include "replication/pgoutput.h"
#include "replication/slot.h"
#include "replication/syncrep.h"
#include "replication/walsender.h"
#include "replication/worker_internal.h"
