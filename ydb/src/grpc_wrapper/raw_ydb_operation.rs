use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct RawOperationParams {
    operation_mode: OperationMode,
    operation_timeout: crate::grpc_wrapper::raw_common_types::Duration,
    cancel_after: crate::grpc_wrapper::raw_common_types::Duration,
    labels: HashMap<String, String>,
}

impl RawOperationParams {
    pub fn new_with_timeouts(
        operation_timeout: std::time::Duration,
        cancel_after: std::time::Duration,
    ) -> Self {
        Self {
            operation_mode: OperationMode::Sync,
            operation_timeout: operation_timeout.into(),
            cancel_after: cancel_after.into(),
            labels: Default::default(),
        }
    }

    pub fn new_with_timeout(timeout: std::time::Duration) -> Self {
        Self::new_with_timeouts(timeout, timeout)
    }
}

impl From<RawOperationParams> for ydb_grpc::ydb_proto::operations::OperationParams {
    fn from(params: RawOperationParams) -> Self {
        Self {
            operation_mode: params.operation_mode.into(),
            operation_timeout: None,
            cancel_after: None,
            labels: params.labels,
            report_cost_info: ydb_grpc::ydb_proto::feature_flag::Status::Unspecified.into(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum OperationMode {
    _Unspecified,
    Sync,
    _Async,
}

use ydb_grpc::ydb_proto::operations::operation_params::OperationMode as GrpcOperationMode;
impl From<OperationMode> for i32 {
    fn from(mode: OperationMode) -> Self {
        let val = match mode {
            OperationMode::_Unspecified => GrpcOperationMode::Unspecified,
            OperationMode::Sync => GrpcOperationMode::Sync,
            OperationMode::_Async => GrpcOperationMode::Async,
        };
        val as i32
    }
}
