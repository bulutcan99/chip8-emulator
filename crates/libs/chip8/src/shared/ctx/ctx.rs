use crate::shared::ctx::error::Error;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: Uuid,
    conv_id: Option<i64>,
}

impl Ctx {
    pub fn root_ctx() -> Self {
        Ctx {
            user_id: Uuid::nil(),
            conv_id: None,
        }
    }

    pub fn new(user_id: Uuid) -> Result<Self> {
        if user_id == Uuid::nil() {
            Err(Error::CtxCannotNewRootCtx)
        } else {
            Ok(Self {
                user_id,
                conv_id: None,
            })
        }
    }

    pub fn add_conv_id(&self, conv_id: i64) -> Ctx {
        let mut ctx = self.clone();
        ctx.conv_id = Some(conv_id);
        ctx
    }
}

impl Ctx {
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn conv_id(&self) -> Option<i64> {
        self.conv_id
    }
}
