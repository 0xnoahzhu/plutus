use crate::db::{Db, Result};
use crate::models::Sector;

pub async fn list(db: &Db) -> Result<Vec<Sector>> {
    db.with(async |d| Sector::all().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn get(db: &Db, code: &str) -> Result<Option<Sector>> {
    let code = code.to_string();
    db.with(async |d| Sector::filter_by_code(code).first().exec(d).await)
        .await
        .map_err(Into::into)
}

pub async fn upsert(
    db: &Db,
    code: &str,
    name: &str,
    parent_code: Option<&str>,
    scheme: &str,
) -> Result<Sector> {
    let code_owned = code.to_string();
    if let Some(existing) = db
        .with(async |d| Sector::filter_by_code(code_owned).first().exec(d).await)
        .await?
    {
        let name = name.to_string();
        let parent = parent_code.map(str::to_string);
        let scheme = scheme.to_string();
        db.with(async |d| {
            let mut e = existing;
            e.update().name(name).parent_code(parent).scheme(scheme).exec(d).await
        })
        .await?;
    } else {
        let code = code.to_string();
        let name = name.to_string();
        let parent = parent_code.map(str::to_string);
        let scheme = scheme.to_string();
        db.with(async |d| {
            toasty::create!(Sector {
                code: code,
                name: name,
                parent_code: parent,
                scheme: scheme,
            })
            .exec(d)
            .await
        })
        .await?;
    }
    let code = code.to_string();
    Ok(db
        .with(async |d| Sector::filter_by_code(code).first().exec(d).await)
        .await?
        .expect("just upserted"))
}
