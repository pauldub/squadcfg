use itertools::Itertools;
use pest::{self, Parser};

#[derive(Debug)]
pub enum GroupPermission {
    Unknown,
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
    fn from_string(value: &str) -> Self {
        match value {
            "startbote" => Self::StartVote,
            "changemap" => Self::ChangeMap,
            "pause" => Self::Pause,
            "cheat" => Self::Cheat,
            "private" => Self::Private,
            "balance" => Self::Balance,
            "chat" => Self::Chat,
            "kick" => Self::Kick,
            "ban" => Self::Ban,
            "config" => Self::Config,
            "cameraman" => Self::Cameraman,
            "immunity" => Self::Immunity,
            "manageserver" => Self::ManageServer,
            "featuretest" => Self::FeatureTest,
            "reserve" => Self::Reserve,
            "demos" => Self::Demos,
            "debug" => Self::Debug,
            "teamchange" => Self::TeamChange,
            "forceteamchange" => Self::ForceTeamChange,
            "canseedadminchat" => Self::CanSeedAdminChat,
            _ => Self::Unknown,
        }
    }

    fn to_whitelist(&self) -> String {
        let value = match self {
            Self::Unknown => "",
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

#[derive(Debug)]
pub struct Group {
    name: String,
    permissions: Vec<GroupPermission>,
    span: Span,
}

impl Group {
    pub fn new(name: &str, permissions: Vec<GroupPermission>) -> Self {
        Self {
            name: name.to_string(),
            permissions,
            span: Span::default(),
        }
    }

    fn set_span(&mut self, span: Span) {
        self.span = span;
    }

    fn to_whitelist(&self) -> String {
        let permissions = self
            .permissions
            .iter()
            .map(GroupPermission::to_whitelist)
            .collect::<Vec<String>>()
            .join(",");

        format!("Group={}:{}", self.name, permissions)
    }
}

#[derive(Debug)]
pub struct Player {
    steam_id: u64,
    group: String,
    comment: String,
    span: Span,
}

impl Player {
    fn new(steam_id: u64, group: &str, comment: &str) -> Self {
        Self {
            steam_id,
            group: group.to_owned(),
            comment: comment.to_owned(),
            span: Span::default(),
        }
    }

    fn set_span(&mut self, span: Span) {
        self.span = span;
    }

    fn to_whitelist(&self) -> String {
        format!("Admin={}:{} // {}", self.steam_id, self.group, self.comment)
    }
}

#[derive(Debug)]
pub struct Whitelist {
    groups: Vec<Group>,
    players: Vec<Player>,
}

impl Whitelist {
    fn new(groups: Vec<Group>, players: Vec<Player>) -> Self {
        Self { groups, players }
    }

    fn to_string(&self) -> String {
        let groups = self
            .groups
            .iter()
            .map(Group::to_whitelist)
            .collect::<Vec<String>>()
            .join("\n");

        let players = self
            .players
            .iter()
            .group_by(|p| p.group.as_str())
            .into_iter()
            .map(|(group_name, players)| {
                let group_players = players
                    .into_iter()
                    .map(Player::to_whitelist)
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("// {}\n{}", group_name, group_players)
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        format!("{}\n\n{}", groups, players)
    }
}

#[derive(Debug, Default)]
struct Span {
    pub start: usize,
    pub end: usize,
}

impl<'a> From<pest::Span<'a>> for Span {
    fn from(span: pest::Span<'a>) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

#[derive(Parser)]
#[grammar = "whitelist.pest"]
struct WhitelistParser {}

fn parse_group(pair: pest::iterators::Pair<Rule>) -> Group {
    let mut name: &str = "";
    let mut permissions: Vec<GroupPermission> = vec![];
    let span: Span = pair.as_span().into();

    for group_pair in pair.into_inner() {
        match group_pair.as_rule() {
            Rule::group_name => {
                name = group_pair.as_str();
            }
            Rule::permissions => {
                for permission_pair in group_pair.into_inner() {
                    let permission = GroupPermission::from_string(permission_pair.as_str());

                    permissions.push(permission);
                }
            }
            _ => {}
        }
    }

    let mut group = Group::new(name, permissions);
    group.set_span(span);
    group
}

fn parse_player(pair: pest::iterators::Pair<Rule>) -> Player {
    let mut steam_id: u64 = 0;
    let mut group_name: &str = "";
    let mut comment: &str = "";
    let span: Span = pair.as_span().into();

    for player_pair in pair.into_inner() {
        match player_pair.as_rule() {
            Rule::player_name => {
                steam_id = player_pair.as_str().parse::<u64>().unwrap_or(0);
            }
            Rule::group_name => {
                group_name = player_pair.as_str();
            }
            Rule::comment => {
                for comment_pair in player_pair.into_inner() {
                    comment = comment_pair.as_str();
                }
            }
            _ => {}
        }
    }

    let mut player = Player::new(steam_id, group_name, comment);
    player.set_span(span);
    player
}

pub fn parse_whitelist(text: &str) -> Result<Whitelist, pest::error::Error<Rule>> {
    let pairs = WhitelistParser::parse(Rule::root, text)?;
    let mut groups: Vec<Group> = vec![];
    let mut players: Vec<Player> = vec![];

    for pair in pairs {
        for rule_pair in pair.into_inner().flatten() {
            match rule_pair.as_rule() {
                Rule::group => groups.push(parse_group(rule_pair)),
                Rule::player => players.push(parse_player(rule_pair)),
                _ => {}
            }
        }
    }

    Ok(Whitelist::new(groups, players))
}

#[cfg(test)]
mod tests {
    use super::{Group, GroupPermission, Player, Whitelist};

    #[test]
    fn it_encodes_a_group_permission() {
        assert_eq!(GroupPermission::TeamChange.to_whitelist(), "teamchange");
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

        assert_eq!(
            group.to_whitelist(),
            "Group=Moderator:changemap,chat,kick,ban"
        );
    }

    #[test]
    fn it_encodes_a_player() {
        let group = Group::new(
            "Moderator",
            vec![
                GroupPermission::ChangeMap,
                GroupPermission::Chat,
                GroupPermission::Kick,
                GroupPermission::Ban,
            ],
        );

        let player = Player::new(76561115695178, &group.name, "Player 1");

        assert_eq!(
            player.to_whitelist(),
            "Admin=76561115695178:Moderator // Player 1"
        );
    }

    #[test]
    fn it_formats_a_whitelist() {
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

        let whitelist = Whitelist::new(
            vec![super_admin, admin, moderator, whitelist],
            vec![
                Player::new(76561115695178, "Moderator", "Player 5"),
                Player::new(8915618948911, "Moderator", "Player 4"),
                Player::new(7894591951519, "Admin", "Player 3"),
                Player::new(7895365435431, "Admin", "Player 8792"),
                Player::new(7984591565611, "SuperAdmin", "Player 2"),
                Player::new(7917236241624, "SuperAdmin", "Player 1"),
                Player::new(7984591565611, "Whitelist", "Player 123"),
                Player::new(7984591565523, "Whitelist", "Player 156"),
            ],
        );

        assert_eq!(whitelist.to_string(), "Group=SuperAdmin:changemap,cheat,private,balance,chat,kick,ban,config,cameraman,debug,pause
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

    #[test]
    fn it_parses_a_whitelist() {
        let whitelist_text = "Group=SuperAdmin:changemap,cheat,private,balance,chat,kick,ban,config,cameraman,debug,pause
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
Admin=7984591565523:Whitelist // Player 156";

        let whitelist = super::parse_whitelist(whitelist_text).unwrap();

        assert_eq!(whitelist.to_string(), whitelist_text);
    }
}
