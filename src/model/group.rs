use serde::{Deserialize, Serialize};
use yew::AttrValue;

use super::{
    friend::{Friend, FriendStatus},
    user::User,
    ItemInfo, RightContentType,
};

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct GroupFromServer {
    pub id: AttrValue,
    pub owner: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub description: AttrValue,
    pub announcement: AttrValue,
    pub create_time: i64,
    pub update_time: i64,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct Group {
    pub id: AttrValue,
    pub owner: AttrValue,
    pub avatar: AttrValue,
    pub name: AttrValue,
    pub create_time: i64,
    pub update_time: i64,
    pub description: AttrValue,
    pub announcement: AttrValue,
    // mark this group if deleted, local only
    #[serde(default)]
    pub deleted: bool,
}

impl From<GroupFromServer> for Group {
    fn from(value: GroupFromServer) -> Self {
        Self {
            id: value.id,
            owner: value.owner,
            avatar: value.avatar,
            name: value.name,
            create_time: value.create_time,
            update_time: value.update_time,
            description: value.description,
            announcement: value.announcement,
            deleted: false,
        }
    }
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GroupRequest {
    pub id: String,
    pub owner: String,
    pub avatar: String,
    pub group_name: String,
    pub members_id: Vec<String>,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GroupDelete {
    pub user_id: String,
    pub group_id: String,
    pub is_dismiss: bool,
}

fn is_zero(id: &i32) -> bool {
    *id == 0
}

/// Group member information
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct GroupMemberFromServer {
    pub age: i32,
    // #[serde(default)]
    pub group_id: AttrValue,
    pub user_id: AttrValue,
    pub group_name: AttrValue,
    // pub account: AttrValue,
    pub avatar: AttrValue,
    pub joined_at: i64,
    pub region: Option<AttrValue>,
    pub gender: AttrValue,
    pub is_friend: bool,
    pub remark: Option<AttrValue>,
    pub signature: AttrValue,
}
/// Group member information
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct GroupMember {
    #[serde(skip_serializing_if = "is_zero")]
    pub id: i32,
    pub age: i32,
    // #[serde(default)]
    pub group_id: AttrValue,
    pub user_id: AttrValue,
    pub group_name: AttrValue,
    // pub account: AttrValue,
    pub avatar: AttrValue,
    pub joined_at: i64,
    pub region: Option<AttrValue>,
    pub gender: AttrValue,
    pub is_friend: bool,
    pub remark: Option<AttrValue>,
    pub signature: AttrValue,
}

impl From<GroupMemberFromServer> for GroupMember {
    fn from(value: GroupMemberFromServer) -> Self {
        Self {
            id: 0,
            group_id: value.group_id,
            age: value.age,
            user_id: value.user_id,
            group_name: value.group_name,
            avatar: value.avatar,
            joined_at: value.joined_at,
            region: value.region,
            gender: value.gender,
            is_friend: value.is_friend,
            remark: value.remark,
            signature: value.signature,
        }
    }
}

impl From<Friend> for GroupMember {
    fn from(value: Friend) -> Self {
        Self {
            id: 0,
            user_id: value.friend_id,
            group_id: AttrValue::default(),
            group_name: value.name,
            avatar: value.avatar,
            region: value.region,
            gender: value.gender,
            joined_at: chrono::Local::now().timestamp_millis(),
            signature: value.signature,
            age: value.age,
            is_friend: true,
            remark: value.remark,
        }
    }
}

impl From<User> for GroupMember {
    fn from(value: User) -> Self {
        Self {
            id: 0,
            user_id: value.id,
            group_id: AttrValue::default(),
            group_name: value.name,
            avatar: value.avatar,
            region: value.address,
            gender: value.gender,
            joined_at: chrono::Local::now().timestamp_millis(),
            signature: value.signature,
            age: value.age,
            is_friend: false,
            remark: None,
        }
    }
}

impl ItemInfo for GroupMember {
    fn name(&self) -> AttrValue {
        self.group_name.clone()
    }

    fn id(&self) -> AttrValue {
        self.user_id.clone()
    }

    fn get_type(&self) -> RightContentType {
        RightContentType::Group
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }

    fn time(&self) -> i64 {
        self.joined_at
    }

    fn remark(&self) -> Option<AttrValue> {
        None
    }

    fn signature(&self) -> AttrValue {
        self.signature.clone()
    }

    fn region(&self) -> Option<AttrValue> {
        self.region.clone()
    }

    fn owner(&self) -> AttrValue {
        self.user_id.clone()
    }

    fn status(&self) -> FriendStatus {
        FriendStatus::Accepted
    }
}

impl ItemInfo for Group {
    fn name(&self) -> AttrValue {
        self.name.clone()
    }

    fn id(&self) -> AttrValue {
        self.id.clone()
    }

    fn get_type(&self) -> RightContentType {
        RightContentType::Group
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }

    fn time(&self) -> i64 {
        self.create_time
    }

    fn remark(&self) -> Option<AttrValue> {
        if self.announcement.is_empty() {
            None
        } else {
            Some(self.announcement.clone())
        }
    }

    fn signature(&self) -> AttrValue {
        self.description.clone()
    }

    fn region(&self) -> Option<AttrValue> {
        None
    }

    fn owner(&self) -> AttrValue {
        self.owner.clone()
    }

    fn status(&self) -> FriendStatus {
        if self.deleted {
            FriendStatus::Blacked
        } else {
            FriendStatus::Accepted
        }
    }
}
