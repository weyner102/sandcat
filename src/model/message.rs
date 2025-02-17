use log::debug;
use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::friend::{FriendShipRequest, FriendShipWithUser, FriendshipWithUser4Response};
use crate::model::ContentType;
use crate::pb::message::{Msg as PbMsg, MsgType};
use crate::{pb, utils};

use super::friend::Friend;
use super::group::{GroupFromServer, GroupMemberFromServer};

pub(crate) const DEFAULT_HELLO_MESSAGE: &str = "I've accepted your friend request. Now let's chat!";

fn is_zero(id: &i32) -> bool {
    *id == 0
}

/// 消息表，要不要每个用户对应一个表？
/// 表名由message+user_id组成
///
/// 由于indexeddb只能在onupgrade中建表，不能动态创建，所以消息只能存到一张表中
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Message {
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub id: i32,
    pub seq: i64,
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    // 是MessageType类型，需要做转换
    // pub msg_type: MessageType,
    #[serde(default)]
    pub content_type: ContentType,
    // 如果是文件类型，那么content存储文件的路径
    pub content: AttrValue,
    #[serde(default)]
    pub create_time: i64,
    #[serde(default)]
    pub send_time: i64,
    pub send_status: SendStatus,
    // pub update_time: String,
    #[serde(default)]
    pub is_read: bool,
    #[serde(default)]
    pub is_self: bool,
    // 是否删除字段可以只存储在服务端
    // pub is_delete: bool,
    #[serde(skip)]
    pub file_content: AttrValue,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub enum SendStatus {
    #[default]
    Sending,
    Success,
    Failed,
}

impl From<InviteCancelMsg> for Message {
    fn from(value: InviteCancelMsg) -> Self {
        let content_type = match value.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        Message {
            id: 0,
            seq: value.seq,
            local_id: value.local_id,
            server_id: value.server_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content: AttrValue::from("已经取消"),
            create_time: value.create_time,
            send_time: value.send_time,
            send_status: value.send_status,
            is_read: false,
            is_self: value.is_self,
            file_content: Default::default(),
        }
    }
}

impl From<InviteAnswerMsg> for Message {
    fn from(value: InviteAnswerMsg) -> Self {
        let content_type = match value.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        let content = if value.agree {
            AttrValue::from("一接通")
        } else {
            AttrValue::from("已经拒绝")
        };
        Message {
            id: 0,
            seq: value.seq,
            local_id: value.local_id,
            server_id: value.server_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content,
            create_time: value.create_time,
            send_time: value.send_time,
            send_status: value.send_status,
            is_read: false,
            is_self: value.is_self,
            file_content: Default::default(),
        }
    }
}

impl From<InviteNotAnswerMsg> for Message {
    fn from(value: InviteNotAnswerMsg) -> Self {
        let content_type = match value.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        let content = AttrValue::from("NOT ANSWER");
        Message {
            id: 0,
            seq: value.seq,
            local_id: value.local_id,
            server_id: value.server_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content,
            create_time: value.create_time,
            send_time: value.send_time,
            send_status: value.send_status,
            is_read: false,
            is_self: value.is_self,
            file_content: Default::default(),
        }
    }
}

impl From<Hangup> for Message {
    fn from(value: Hangup) -> Self {
        let content_type = match value.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        let content = AttrValue::from(utils::format_milliseconds(value.sustain));

        Message {
            id: 0,
            seq: value.seq,
            local_id: value.local_id,
            server_id: value.server_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content,
            create_time: value.create_time,
            send_time: value.send_time,
            send_status: value.send_status,
            is_read: false,
            is_self: value.is_self,
            file_content: Default::default(),
        }
    }
}

impl Message {
    pub fn from_hangup(value: Hangup) -> Self {
        let content_type = match value.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        // 计算时间
        let content = AttrValue::from(utils::format_milliseconds(value.sustain));
        Message {
            id: 0,
            seq: value.seq,
            local_id: value.local_id,
            server_id: value.server_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content,
            create_time: value.create_time,
            send_time: value.send_time,
            send_status: value.send_status,
            is_read: false,
            is_self: value.is_self,
            file_content: Default::default(),
        }
    }
    pub fn from_not_answer(msg: InviteNotAnswerMsg) -> Self {
        let content_type = match msg.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };

        Self {
            id: 0,
            seq: msg.seq,
            local_id: msg.local_id,
            server_id: msg.server_id,
            send_id: msg.send_id,
            friend_id: msg.friend_id,
            content_type,
            content: AttrValue::from("Not Answer"),
            create_time: msg.create_time,
            send_time: msg.send_time,
            send_status: msg.send_status,
            is_read: msg.is_self,
            is_self: msg.is_self,
            file_content: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GroupInvitation {
    pub info: Option<GroupFromServer>,
    pub members: Vec<GroupMemberFromServer>,
}

pub type MessageID = String;
pub type GroupID = String;
pub type UserID = String;
pub type FriendID = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Msg {
    Single(Message),
    Group(GroupMsg),
    // GroupInvitation(GroupInvitation),
    SendRelationshipReq(FriendShipRequest),
    RecRelationship((FriendShipWithUser, Sequence)),
    RecRelationshipDel((FriendID, Sequence)),
    RelationshipRes((Friend, Sequence)),
    ReadNotice(ReadNotice),
    SingleDeliveredNotice(MessageID),
    FriendshipDeliveredNotice(MessageID),
    OfflineSync(Message),
    SingleCall(SingleCall),
    ServerRecResp(ServerResponse), // GroupInvitationReceived((UserID, GroupID)),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum RespMsgType {
    #[default]
    Single,
    Group,
}
/// server received message and return the result(success/failed)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ServerResponse {
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_status: SendStatus,
    pub err_msg: Option<AttrValue>,
    pub resp_msg_type: RespMsgType,
}

impl Default for Msg {
    fn default() -> Self {
        Self::Single(Message::default())
    }
}

pub type Sequence = i64;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GroupMsg {
    Message(Message),
    Invitation((GroupInvitation, Sequence)),
    MemberExit((UserID, GroupID, Sequence)),
    Dismiss((GroupID, Sequence)),
    DismissOrExitReceived((UserID, GroupID)),
    InvitationReceived((UserID, GroupID)),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SingleCall {
    Offer(Offer),
    Invite(InviteMsg),
    InviteCancel(InviteCancelMsg),
    NotAnswer(InviteNotAnswerMsg),
    InviteAnswer(InviteAnswerMsg),
    Agree(Agree),
    HangUp(Hangup),
    NewIceCandidate(Candidate),
}

impl Default for SingleCall {
    fn default() -> Self {
        Self::Offer(Offer::default())
    }
}
#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Offer {
    pub sdp: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
}

#[derive(Debug, Default, Serialize)]
pub struct InviteInfo {
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub invite_type: InviteType,
    pub start_time: i64,
    pub end_time: i64,
    pub connected: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteMsg {
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub invite_type: InviteType,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteNotAnswerMsg {
    pub seq: i64,
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub send_time: i64,
    pub invite_type: InviteType,
    pub send_status: SendStatus,
    #[serde(default)]
    pub is_self: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteCancelMsg {
    pub seq: i64,
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub send_time: i64,
    pub invite_type: InviteType,
    pub send_status: SendStatus,
    #[serde(default)]
    pub is_self: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum InviteType {
    Video,
    #[default]
    Audio,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteAnswerMsg {
    pub seq: i64,
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub send_time: i64,
    pub agree: bool,
    pub invite_type: InviteType,
    pub send_status: SendStatus,
    // 主要区分发起端，因为接收端永远都是false不需要处理
    #[serde(default)]
    pub is_self: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Candidate {
    pub candidate: AttrValue,
    pub sdp_mid: Option<String>,
    pub sdp_m_index: Option<u16>,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Agree {
    pub sdp: Option<String>,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Hangup {
    pub seq: i64,
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub send_time: i64,
    pub invite_type: InviteType,
    pub sustain: i64,
    pub send_status: SendStatus,
    #[serde(default)]
    pub is_self: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    pub send_id: String,
    pub friend_id: String,
    pub status: RelationStatus,
    pub create_time: i64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationStatus {
    #[default]
    Apply,
    Agree,
    Deny,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReadNotice {
    pub msg_ids: Vec<String>,
    pub send_id: String,
    pub friend_id: String,
    pub create_time: i64,
}

impl TryFrom<pb::message::Msg> for Message {
    type Error = String;

    fn try_from(value: pb::message::Msg) -> Result<Self, Self::Error> {
        let msg_type = MsgType::try_from(value.msg_type).map_err(|e| e.to_string())?;
        let friend_id = if msg_type == MsgType::GroupMsg {
            value.group_id.into()
        } else {
            value.receiver_id.into()
        };
        let status: ContentType = value.content_type.into();
        let send_status = if status == ContentType::Error {
            SendStatus::Failed
        } else {
            SendStatus::Success
        };
        Ok(Self {
            id: 0,
            seq: value.seq,
            local_id: value.local_id.into(),
            server_id: value.server_id.into(),
            send_id: value.send_id.into(),
            friend_id,
            content_type: ContentType::from(value.content_type),
            content: String::from_utf8(value.content)
                .map_err(|e| e.to_string())?
                .into(),
            create_time: value.create_time,
            send_time: value.send_time,
            send_status,
            is_read: false,
            is_self: false,
            file_content: AttrValue::default(),
        })
    }
}

pub fn convert_server_msg(msg: PbMsg) -> Result<Msg, String> {
    debug!("convert msg: {:?}", msg);
    let msg_type = MsgType::try_from(msg.msg_type).unwrap();
    match msg_type {
        MsgType::SingleMsg => Ok(Msg::Single(Message::try_from(msg)?)),
        MsgType::GroupMsg => Ok(Msg::Group(GroupMsg::Message(Message::try_from(msg)?))),
        MsgType::GroupInvitation => {
            // decode content
            let info = bincode::deserialize(&msg.content).map_err(|e| e.to_string())?;
            Ok(Msg::Group(GroupMsg::Invitation((info, msg.seq))))
        }
        MsgType::GroupInviteNew => todo!(),
        MsgType::GroupMemberExit => Ok(Msg::Group(GroupMsg::MemberExit((
            msg.send_id,
            msg.group_id,
            msg.seq,
        )))),
        MsgType::GroupDismiss => Ok(Msg::Group(GroupMsg::Dismiss((msg.group_id, msg.seq)))),
        MsgType::GroupDismissOrExitReceived => todo!(),
        MsgType::GroupInvitationReceived => todo!(),
        MsgType::GroupUpdate => todo!(),
        MsgType::FriendApplyReq => {
            // decode content
            let info: FriendshipWithUser4Response =
                bincode::deserialize(&msg.content).map_err(|e| e.to_string())?;
            Ok(Msg::RecRelationship((
                FriendShipWithUser::from(info),
                msg.seq,
            )))
        }
        MsgType::FriendApplyResp => {
            // decode content
            let info = bincode::deserialize(&msg.content).map_err(|e| e.to_string())?;
            Ok(Msg::RelationshipRes((info, msg.seq)))
        }
        MsgType::SingleCallInvite => {
            let invite_type = get_invite_type(msg.content_type)?;
            Ok(Msg::SingleCall(SingleCall::Invite(InviteMsg {
                local_id: msg.local_id.into(),
                server_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.send_time,
                invite_type,
            })))
        }
        MsgType::RejectSingleCall => {
            let invite_type = get_invite_type(msg.content_type)?;
            Ok(Msg::SingleCall(SingleCall::InviteAnswer(InviteAnswerMsg {
                seq: msg.seq,
                local_id: msg.local_id.into(),
                server_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.create_time,
                invite_type,
                agree: false,
                is_self: false,
                send_time: msg.send_time,
                send_status: SendStatus::Success,
            })))
        }
        MsgType::SingleCallInviteNotAnswer => {
            let invite_type = get_invite_type(msg.content_type)?;
            Ok(Msg::SingleCall(SingleCall::NotAnswer(InviteNotAnswerMsg {
                seq: msg.seq,
                local_id: msg.local_id.into(),
                server_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.create_time,
                invite_type,
                is_self: false,
                send_time: msg.send_time,
                send_status: SendStatus::Success,
            })))
        }
        MsgType::SingleCallInviteCancel => {
            let invite_type = get_invite_type(msg.content_type)?;
            Ok(Msg::SingleCall(SingleCall::InviteCancel(InviteCancelMsg {
                seq: msg.seq,
                local_id: msg.local_id.into(),
                server_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.create_time,
                invite_type,
                is_self: false,
                send_time: msg.send_time,
                send_status: SendStatus::Success,
            })))
        }
        MsgType::SingleCallOffer => Ok(Msg::SingleCall(SingleCall::Offer(Offer {
            send_id: msg.send_id.into(),
            friend_id: msg.receiver_id.into(),
            create_time: msg.send_time,
            sdp: msg.sdp.ok_or_else(|| String::from("sdp is empty"))?.into(),
        }))),
        MsgType::Hangup => Ok(Msg::SingleCall(SingleCall::HangUp(Hangup {
            seq: msg.seq,
            local_id: msg.local_id.into(),
            server_id: msg.server_id.into(),
            send_id: msg.send_id.into(),
            friend_id: msg.receiver_id.into(),
            create_time: msg.create_time,
            send_time: msg.send_time,
            invite_type: InviteType::Audio,
            sustain: i64::from_be_bytes(
                msg.content
                    .try_into()
                    .map_err(|_| String::from("sustain convert error"))?,
            ),
            is_self: false,
            send_status: SendStatus::Success,
        }))),
        MsgType::AgreeSingleCall => {
            let invite_type = get_invite_type(msg.content_type)?;
            Ok(Msg::SingleCall(SingleCall::InviteAnswer(InviteAnswerMsg {
                seq: msg.seq,
                local_id: msg.local_id.into(),
                server_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.create_time,
                invite_type,
                agree: true,
                is_self: false,
                send_time: msg.send_time,
                send_status: SendStatus::Success,
            })))
        }
        MsgType::ConnectSingleCall => Ok(Msg::SingleCall(SingleCall::Agree(Agree {
            send_id: msg.send_id.into(),
            friend_id: msg.receiver_id.into(),
            create_time: msg.send_time,
            sdp: msg.sdp,
        }))),
        MsgType::Candidate => Ok(Msg::SingleCall(SingleCall::NewIceCandidate(Candidate {
            candidate: String::from_utf8(msg.content)
                .map_err(|e| e.to_string())?
                .into(),
            sdp_mid: msg.sdp_mid,
            sdp_m_index: msg.sdp_m_index.map(|i| i as u16),
            send_id: msg.send_id.into(),
            friend_id: msg.receiver_id.into(),
            create_time: msg.send_time,
        }))),
        MsgType::Read => todo!(),
        MsgType::MsgRecResp => {
            let msg_type = if msg.group_id.is_empty() {
                RespMsgType::Single
            } else {
                RespMsgType::Group
            };
            Ok(Msg::ServerRecResp(ServerResponse {
                local_id: msg.local_id.into(),
                send_status: if msg.content_type == ContentType::Error as i32 {
                    SendStatus::Failed
                } else {
                    SendStatus::Success
                },
                err_msg: None,
                server_id: msg.server_id.into(),
                resp_msg_type: msg_type,
            }))
        }
        MsgType::Notification => todo!(),
        MsgType::Service => todo!(),
        MsgType::FriendshipReceived => todo!(),
        MsgType::FriendDelete => Ok(Msg::RecRelationshipDel((msg.send_id, msg.seq))),
    }
}

fn get_invite_type(t: i32) -> Result<InviteType, String> {
    match ContentType::from(t) {
        ContentType::VideoCall => Ok(InviteType::Video),
        ContentType::AudioCall => Ok(InviteType::Audio),
        _ => Err("Invalid content type".to_string()),
    }
}
impl From<Msg> for PbMsg {
    fn from(value: Msg) -> Self {
        match value {
            Msg::Single(msg) => PbMsg {
                msg_type: MsgType::SingleMsg as i32,
                local_id: msg.local_id.as_str().into(),
                send_id: msg.send_id.as_str().into(),
                receiver_id: msg.friend_id.as_str().into(),
                create_time: msg.create_time,
                content_type: msg.content_type as i32,
                content: msg.content.as_bytes().to_vec(),
                ..Default::default()
            },
            Msg::Group(group_msg) => {
                let mut pb_msg = PbMsg::default();
                match group_msg {
                    GroupMsg::Message(msg) => {
                        pb_msg.msg_type = MsgType::GroupMsg as i32;
                        pb_msg.local_id = msg.local_id.as_str().into();
                        pb_msg.send_id = msg.send_id.as_str().into();
                        pb_msg.receiver_id = msg.friend_id.to_string();
                        pb_msg.group_id = msg.friend_id.to_string();
                        pb_msg.create_time = msg.create_time;
                        pb_msg.content_type = msg.content_type as i32;
                        pb_msg.content = msg.content.as_bytes().to_vec();
                    }
                    GroupMsg::Invitation(info) => {
                        pb_msg.msg_type = MsgType::GroupInvitation as i32;
                        pb_msg.content = bincode::serialize(&info).unwrap();
                    }
                    GroupMsg::MemberExit((send_id, group_id, _)) => {
                        pb_msg.msg_type = MsgType::GroupMemberExit as i32;
                        pb_msg.send_id = send_id.to_string();
                        pb_msg.receiver_id = group_id.to_string();
                        pb_msg.group_id = group_id.to_string();
                    }
                    GroupMsg::Dismiss((group_id, _)) => {
                        pb_msg.msg_type = MsgType::GroupDismiss as i32;
                        pb_msg.receiver_id = group_id.to_string();
                        pb_msg.group_id = group_id.to_string();
                    }
                    GroupMsg::DismissOrExitReceived(_) => {}
                    GroupMsg::InvitationReceived(_) => {}
                }
                pb_msg
            }
            Msg::SingleCall(call) => {
                let mut pb_msg = PbMsg::default();
                match call {
                    SingleCall::Invite(invite) => {
                        pb_msg.msg_type = MsgType::SingleCallInvite as i32;
                        pb_msg.local_id = invite.server_id.as_str().into();
                        pb_msg.send_id = invite.send_id.as_str().into();
                        pb_msg.receiver_id = invite.friend_id.as_str().into();
                        pb_msg.create_time = invite.create_time;
                        pb_msg.content_type = match invite.invite_type {
                            InviteType::Video => ContentType::VideoCall as i32,
                            InviteType::Audio => ContentType::AudioCall as i32,
                        };
                    }
                    SingleCall::InviteAnswer(answer) => {
                        pb_msg.msg_type = MsgType::RejectSingleCall as i32;
                        if answer.agree {
                            pb_msg.msg_type = MsgType::AgreeSingleCall as i32;
                        }
                        pb_msg.local_id = answer.local_id.as_str().into();
                        pb_msg.send_id = answer.send_id.as_str().into();
                        pb_msg.receiver_id = answer.friend_id.as_str().into();
                        pb_msg.create_time = answer.create_time;
                        pb_msg.content_type = match answer.invite_type {
                            InviteType::Video => ContentType::VideoCall as i32,
                            InviteType::Audio => ContentType::AudioCall as i32,
                        };
                    }
                    SingleCall::NotAnswer(not_answer) => {
                        pb_msg.msg_type = MsgType::SingleCallInviteNotAnswer as i32;
                        pb_msg.local_id = not_answer.local_id.as_str().into();
                        pb_msg.send_id = not_answer.send_id.as_str().into();
                        pb_msg.receiver_id = not_answer.friend_id.as_str().into();
                        pb_msg.create_time = not_answer.create_time;
                        pb_msg.content_type = match not_answer.invite_type {
                            InviteType::Video => ContentType::VideoCall as i32,
                            InviteType::Audio => ContentType::AudioCall as i32,
                        };
                    }
                    SingleCall::InviteCancel(cancel) => {
                        pb_msg.msg_type = MsgType::SingleCallInviteCancel as i32;
                        pb_msg.local_id = cancel.local_id.as_str().into();
                        pb_msg.send_id = cancel.send_id.as_str().into();
                        pb_msg.receiver_id = cancel.friend_id.as_str().into();
                        pb_msg.create_time = cancel.create_time;
                        pb_msg.content_type = match cancel.invite_type {
                            InviteType::Video => ContentType::VideoCall as i32,
                            InviteType::Audio => ContentType::AudioCall as i32,
                        };
                    }
                    SingleCall::Offer(offer) => {
                        pb_msg.msg_type = MsgType::SingleCallOffer as i32;
                        pb_msg.send_id = offer.send_id.as_str().into();
                        pb_msg.receiver_id = offer.friend_id.as_str().into();
                        pb_msg.create_time = offer.create_time;
                        pb_msg.sdp = Some(offer.sdp.to_string());
                    }
                    SingleCall::Agree(agree) => {
                        pb_msg.msg_type = MsgType::ConnectSingleCall as i32;
                        pb_msg.send_id = agree.send_id.as_str().into();
                        pb_msg.receiver_id = agree.friend_id.as_str().into();
                        pb_msg.create_time = agree.create_time;
                        pb_msg.sdp = agree.sdp;
                    }
                    SingleCall::HangUp(hangup) => {
                        pb_msg.msg_type = MsgType::Hangup as i32;
                        pb_msg.send_id = hangup.send_id.as_str().into();
                        pb_msg.receiver_id = hangup.friend_id.as_str().into();
                        pb_msg.create_time = hangup.create_time;
                        pb_msg.content = hangup.sustain.to_be_bytes().to_vec();
                        pb_msg.local_id = hangup.local_id.as_str().into();
                    }
                    SingleCall::NewIceCandidate(candidate) => {
                        pb_msg.msg_type = MsgType::Candidate as i32;
                        pb_msg.send_id = candidate.send_id.as_str().into();
                        pb_msg.receiver_id = candidate.friend_id.as_str().into();
                        pb_msg.create_time = candidate.create_time;
                        pb_msg.sdp_mid = candidate.sdp_mid;
                        pb_msg.sdp_m_index = candidate.sdp_m_index.map(|c| c as i32);
                        pb_msg.content = candidate.candidate.as_bytes().to_vec();
                    }
                }
                pb_msg
            }
            Msg::SendRelationshipReq(msg) => PbMsg {
                msg_type: MsgType::FriendApplyReq as i32,
                content: bincode::serialize(&msg).unwrap(),
                ..Default::default()
            },
            Msg::RecRelationship(_) => PbMsg {
                msg_type: MsgType::FriendApplyReq as i32,
                ..Default::default()
            },
            Msg::RelationshipRes(_) => PbMsg {
                msg_type: MsgType::FriendApplyResp as i32,
                ..Default::default()
            },
            Msg::ReadNotice(_) => PbMsg {
                msg_type: MsgType::Read as i32,
                ..Default::default()
            },

            Msg::SingleDeliveredNotice(_) => PbMsg::default(),
            Msg::FriendshipDeliveredNotice(_) => PbMsg::default(),
            Msg::OfflineSync(_) => PbMsg::default(),
            Msg::ServerRecResp(_) => PbMsg::default(),
            Msg::RecRelationshipDel(_) => PbMsg::default(),
        }
    }
}
