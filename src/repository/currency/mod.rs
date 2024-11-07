use super::{AssetInfo, AssetKind, IRepository};
use crate::database::asset::AssetId;
use crate::error::ServerError;
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize)]
struct Datum {
    name: String,
    symbol: String,
}

static DATA: LazyLock<Vec<Datum>> = LazyLock::new(|| {
    let data: Vec<Datum> =
        serde_json::from_str(include_str!("currency.json")).unwrap();
    data
});

pub struct Repository;

impl IRepository for Repository {
    async fn search(
        &self,
        _: uuid::Uuid,
        prefix: String,
        kinds: Vec<AssetKind>,
    ) -> Result<Vec<AssetInfo>, ServerError> {
        if kinds != vec![AssetKind::Currency] {
            return Err(ServerError::Internal(String::from("Unsupported")));
        }

        let result = DATA
            .iter()
            .filter(|x| x.symbol.starts_with(&prefix))
            .map(|x| AssetInfo {
                id: AssetId::CURRENCY(x.symbol.clone()),
                name: x.name.clone(),
                kind: AssetKind::Currency,
            })
            .collect();
        Ok(result)
    }
}
