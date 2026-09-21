use shared::project::Project;

pub enum ProjectItemEvents {
    OpenProject(Project),
    CloseProject(i64),
}
