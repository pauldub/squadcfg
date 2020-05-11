use itertools::Itertools;

pub enum GroupPermission {
    StartVote,
    ChangeMap,
    Pause,
    Cheat,
    Private,
    Balance,
    Chat,
    Kick,
    Ban,
    Config,
    Cameraman,
    Immunity,
    ManageServer,
    FeatureTest,
    Reserve,
    Demos,
    Debug,
    TeamChange,
    ForceTeamChange,
    CanSeedAdminChat,
}

impl GroupPermission {
    fn to_config(&self) -> String {
        let value = match self {
            Self::StartVote => "startbote",
            Self::ChangeMap => "changemap",
            Self::Pause => "pause",
            Self::Cheat => "cheat",
            Self::Private => "private",
            Self::Balance => "balance",
            Self::Chat => "chat",
            Self::Kick => "kick",
            Self::Ban => "ban",
            Self::Config => "config",
            Self::Cameraman => "cameraman",
            Self::Immunity => "immunity",
            Self::ManageServer => "manageserver",
            Self::FeatureTest => "featuretest",
            Self::Reserve => "reserve",
            Self::Demos => "demos",
            Self::Debug => "debug",
            Self::TeamChange => "teamchange",
            Self::ForceTeamChange => "forceteamchange",
            Self::CanSeedAdminChat => "canseedadminchat",
        };

        value.to_owned()
    }
}

pub struct Group {
    name: String,
    permissions: Vec<GroupPermission>,
}

impl Group {
    pub fn new(name: &str, permissions: Vec<GroupPermission>) -> Self {
        Self {
            name: name.to_string(),
            permissions,
        }
    }

    fn to_config(&self) -> String {
        let permissions = self
            .permissions
            .iter()
            .map(GroupPermission::to_config)
            .collect::<Vec<String>>()
            .join(",");

        format!("Group={}:{}", self.name, permissions)
    }
}

pub struct Permission<'a> {
    steam_id: u64,
    group: &'a Group,
    comment: String,
}

impl<'a> Permission<'a> {
    fn new(steam_id: u64, group: &'a Group, comment: &str) -> Self {
        Self {
            steam_id,
            group,
            comment: comment.to_owned(),
        }
    }

    fn to_config(&self) -> String {
        format!(
            "Admin={}:{} // {}",
            self.steam_id, self.group.name, self.comment
        )
    }
}

pub struct Config<'a> {
    groups: Vec<&'a Group>,
    permissions: Vec<Permission<'a>>,
}

impl<'a> Config<'a> {
    fn new(groups: Vec<&'a Group>, permissions: Vec<Permission<'a>>) -> Self {
        Self {
            groups,
            permissions,
        }
    }

    fn to_string(&self) -> String {
        let groups = self
            .groups
            .iter()
            .map(|g| Group::to_config(*g))
            .collect::<Vec<String>>()
            .join("\n");

        let permissions = self
            .permissions
            .iter()
            .group_by(|p| p.group.name.as_str())
            .into_iter()
            .map(|(group_name, permissions)| {
                let group_permissions = permissions
                    .into_iter()
                    .map(Permission::to_config)
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("// {}\n{}", group_name, group_permissions)
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        format!("{}\n\n{}", groups, permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, Group, GroupPermission, Permission};

    #[test]
    fn it_encodes_a_group_permission() {
        assert_eq!(GroupPermission::TeamChange.to_config(), "teamchange");
    }

    #[test]
    fn it_encodes_a_group() {
        let group = Group::new(
            "Moderator",
            vec![
                GroupPermission::ChangeMap,
                GroupPermission::Chat,
                GroupPermission::Kick,
                GroupPermission::Ban,
            ],
        );

        assert_eq!(group.to_config(), "Group=Moderator:changemap,chat,kick,ban");
    }

    #[test]
    fn it_encodes_a_permission() {
        let group = Group::new(
            "Moderator",
            vec![
                GroupPermission::ChangeMap,
                GroupPermission::Chat,
                GroupPermission::Kick,
                GroupPermission::Ban,
            ],
        );

        let permission = Permission::new(76561115695178, &group, "Player 1");

        assert_eq!(
            permission.to_config(),
            "Admin=76561115695178:Moderator // Player 1"
        );
    }

    #[test]
    fn it_formats_a_config() {
        let super_admin = Group::new(
            "SuperAdmin",
            vec![
                GroupPermission::ChangeMap,
                GroupPermission::Cheat,
                GroupPermission::Private,
                GroupPermission::Balance,
                GroupPermission::Chat,
                GroupPermission::Kick,
                GroupPermission::Ban,
                GroupPermission::Config,
                GroupPermission::Cameraman,
                GroupPermission::Debug,
                GroupPermission::Pause,
            ],
        );

        let admin = Group::new(
            "Admin",
            vec![
                GroupPermission::ChangeMap,
                GroupPermission::Balance,
                GroupPermission::Chat,
                GroupPermission::Kick,
                GroupPermission::Ban,
                GroupPermission::Cameraman,
                GroupPermission::Pause,
            ],
        );

        let moderator = Group::new(
            "Moderator",
            vec![
                GroupPermission::ChangeMap,
                GroupPermission::Chat,
                GroupPermission::Kick,
                GroupPermission::Ban,
            ],
        );

        let whitelist = Group::new("Whitelist", vec![GroupPermission::Reserve]);

        let config = Config::new(
            vec![&super_admin, &admin, &moderator, &whitelist],
            vec![
                Permission::new(76561115695178, &moderator, "Player 5"),
                Permission::new(8915618948911, &moderator, "Player 4"),
                Permission::new(7894591951519, &admin, "Player 3"),
                Permission::new(7895365435431, &admin, "Player 8792"),
                Permission::new(7984591565611, &super_admin, "Player 2"),
                Permission::new(7917236241624, &super_admin, "Player 1"),
                Permission::new(7984591565611, &whitelist, "Player 123"),
                Permission::new(7984591565523, &whitelist, "Player 156"),
            ],
        );

        assert_eq!(config.to_string(), "Group=SuperAdmin:changemap,cheat,private,balance,chat,kick,ban,config,cameraman,debug,pause
Group=Admin:changemap,balance,chat,kick,ban,cameraman,pause
Group=Moderator:changemap,chat,kick,ban
Group=Whitelist:reserve

// Moderator
Admin=76561115695178:Moderator // Player 5
Admin=8915618948911:Moderator // Player 4

// Admin
Admin=7894591951519:Admin // Player 3
Admin=7895365435431:Admin // Player 8792

// SuperAdmin
Admin=7984591565611:SuperAdmin // Player 2
Admin=7917236241624:SuperAdmin // Player 1

// Whitelist
Admin=7984591565611:Whitelist // Player 123
Admin=7984591565523:Whitelist // Player 156");
    }
}
