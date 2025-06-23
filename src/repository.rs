use crate::entity::Entity;

pub trait Repository<E: Entity>: 'static {
    async fn find(&self, id: &E::SysId) -> anyhow::Result<Option<E>>;

    async fn save(&self, entity: &E) -> anyhow::Result<SaveEffect>;

    async fn update(&self, entity: &E) -> anyhow::Result<UpdateEffect>;

    async fn delete(&self, id: &E::SysId) -> anyhow::Result<DeleteEffect>;
}

#[must_use = "Save effect should be checked"]
pub enum SaveEffect {
    Ok,
    Conflict,
}

#[must_use = "Delete effect should be checked"]
pub enum DeleteEffect {
    Ok,
    NotFound,
}

#[must_use = "Update effect should be checked"]
pub enum UpdateEffect {
    Ok,
    Conflict,
    NotFound,
}

impl UpdateEffect {
    pub fn is_not_found(&self) -> bool {
        matches!(self, UpdateEffect::NotFound)
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, UpdateEffect::Ok)
    }

    pub fn is_effected(&self) -> bool {
        self.is_ok()
    }

    pub fn is_conflict(&self) -> bool {
        matches!(self, UpdateEffect::Conflict)
    }

    pub fn ignore_effect(self) {}
}

impl SaveEffect {
    pub fn is_conflict(&self) -> bool {
        matches!(self, SaveEffect::Conflict)
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, SaveEffect::Ok)
    }

    pub fn is_effected(&self) -> bool {
        self.is_ok()
    }

    pub fn ignore_effect(self) {}
}

impl DeleteEffect {
    pub fn is_not_found(&self) -> bool {
        matches!(self, DeleteEffect::NotFound)
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, DeleteEffect::Ok)
    }

    pub fn is_effected(&self) -> bool {
        self.is_ok()
    }

    pub fn ignore_effect(self) {}
}
