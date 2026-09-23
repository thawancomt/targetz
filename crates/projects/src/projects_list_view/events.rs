use shared::project::Project;

pub enum ProjectListEvents {
    OpenFormView,
    CloseFormView,
    OpenProject(Project),
}
