/// Replica value is used to define replication value
/// It can be a number or auto
use serde::{Serialize, Deserialize};

/// Auto is used to automatically define that the number of replicas in the cluster
/// Number is used to manually set the number of replicas
/// Note: auto will ensure at least 1 replica exists in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub enum ReplicaValue {
  #[serde(rename = "number")]
  Number(i64),
  #[serde(rename = "auto")]
  Auto,
}

/// Cargo replication is used to define the minimum and maximum number of replicas in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoReplication {
  pub min_replicas: Option<i64>,
  pub max_replicas: Option<i64>,
}

/// A cargo is a replicated container
/// CargoConfig is used to define the configuration of the cargo
/// It's used to create a [CargoConfig](CargoConfig)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoConfigPartial {
  pub name: String,
  pub dns_entry: Option<String>,
  pub container: bollard::container::Config<String>,
  pub replication: Option<CargoReplication>,
}

/// A cargo is a replicated container
/// CargoConfig is used to define the configuration of the cargo
/// It's used to create a [CargoConfig](CargoConfig)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "dev", derive(ToSchema))]
pub struct CargoConfig {
  pub key: uuid::Uuid,
  pub name: String,
  pub cargo_key: String,
  pub dns_entry: Option<String>,
  pub container: bollard::container::Config<String>,
  pub replication: Option<CargoReplication>,
}