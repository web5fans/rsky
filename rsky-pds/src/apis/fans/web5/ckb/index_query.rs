use crate::account_manager::helpers::account::{get_first_item, get_fourth_item, get_second_item, get_third_item};
use crate::apis::ApiError;
use crate::auth_verifier::UserDidAuthOptional;
use crate::db::DbConn;
use rocket::serde::json::Json;
use rsky_lexicon::fans::web5::ckb::{
    FirstResult, FourthResult, IndexQueryInput, IndexQueryInputRef, IndexQueryOutput, IndexQueryOutputRef, SecondResult, ThirdResult
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransformedWeb5CreateAccountInput {
    pub handle: String,
    pub did: String,
    pub invite_code: Option<String>,
}

//TODO: Potential for taking advantage of async better
#[tracing::instrument(skip_all)]
#[rocket::post("/xrpc/fans.web5.ckb.indexQuery", format = "json", data = "<body>")]
pub async fn index_query(
    body: Json<IndexQueryInput>,
    _auth: UserDidAuthOptional,
    db: DbConn,
) -> Result<Json<IndexQueryOutput>, ApiError> {
    let input: IndexQueryInput = body.into_inner();

    let output = match input.index {
        IndexQueryInputRef::First(_) => IndexQueryOutput {
            result: IndexQueryOutputRef::First(FirstResult {
                result: get_first_item(&db).await? as usize,
            }),
        },
        IndexQueryInputRef::Second(_) => IndexQueryOutput {
            result: IndexQueryOutputRef::Second(SecondResult {
                result: get_second_item(&db).await? as usize,
            }),
        },
        IndexQueryInputRef::Third(third) => IndexQueryOutput {
            result: IndexQueryOutputRef::Third(ThirdResult {
                result: get_third_item(&db, third.did).await? as usize,
            }),
        },
        IndexQueryInputRef::Fourth(fourth) => IndexQueryOutput {
            result: IndexQueryOutputRef::Fourth(FourthResult {
                result: get_fourth_item(&db, fourth.did).await?,
            }),
        },
    };

    return Ok(Json(output));
}
