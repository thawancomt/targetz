use shared::project::Project;

pub enum ProjectUpdateEvent {
    UpdatedProject(Project),
}
