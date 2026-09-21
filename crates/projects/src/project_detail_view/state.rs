use gpui_kit::{App, AppContext, Entity, Window};
use shared::project::Project;

pub struct ProjectDetailView {
    pub project: Project,
}

impl ProjectDetailView {
    pub fn new(_window: &mut Window, cx: &mut App, project: Project) -> Entity<Self> {
        cx.new(|_cx| Self { project })
    }
}
