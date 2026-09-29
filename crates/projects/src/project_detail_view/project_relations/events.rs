use gpui_kit::EventEmitter;

use crate::project_detail_view::{
    components::stakeholder_item::StakeholderItemEvent,
    project_relations::state::ProjectRelationView,
};

impl EventEmitter<StakeholderItemEvent> for ProjectRelationView {}
