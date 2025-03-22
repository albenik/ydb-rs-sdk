use ydb_grpc::ydb_proto::topic::AutoPartitioningSettings;

#[derive(serde::Serialize)]
pub(crate) struct RawPartitioningSettings {
    pub max_active_partitions: i64,
    pub min_active_partitions: i64,
    pub auto_partitioning_settings: Option<AutoPartitioningSettings>,
}

#[allow(deprecated)]
impl From<RawPartitioningSettings> for ydb_grpc::ydb_proto::topic::PartitioningSettings {
    fn from(value: RawPartitioningSettings) -> Self {
        Self {
            min_active_partitions: value.min_active_partitions,
            max_active_partitions: value.max_active_partitions,
            partition_count_limit: value.max_active_partitions,
            auto_partitioning_settings: value.auto_partitioning_settings,
        }
    }
}
