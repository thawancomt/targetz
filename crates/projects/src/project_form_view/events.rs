use shared::project::Project;

pub enum CreateProjectEvent {
    CreatedProject(Project),
}
