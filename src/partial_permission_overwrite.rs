use serenity::model::{
    permissions::Permissions,
    channel::{PermissionOverwriteType, PermissionOverwrite},
};

#[derive(Copy, Clone)]
pub struct PartialPermissionOverwrite {
    allow: Permissions,
    deny: Permissions,
}

pub fn create_lock_permisson() -> PartialPermissionOverwrite {
    PartialPermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::SEND_MESSAGES,
    }
}

pub fn create_unlock_permisson() -> PartialPermissionOverwrite {
    PartialPermissionOverwrite {
        allow: Permissions::SEND_MESSAGES,
        deny: Permissions::empty(),
    }
}

impl PartialPermissionOverwrite {
    pub fn to_permission_overwrite(&self, kind: PermissionOverwriteType) -> PermissionOverwrite {
        PermissionOverwrite {
            allow: self.allow,
            deny: self.deny,
            kind,
        }
    }
}

