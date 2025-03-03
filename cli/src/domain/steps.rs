use osentities::connection_definition::ConnectionDefinition;

#[derive(Debug)]
pub struct Step {
    question: String,
    key: String,
}

impl Step {
    pub fn new(question: String, key: String) -> Self {
        Self { question, key }
    }

    pub fn question(&self) -> &str {
        &self.question
    }

    pub fn from(conn_def: &ConnectionDefinition) -> Vec<Self> {
        conn_def
            .auth_secrets
            .iter()
            .filter_map(|a| {
                let name = a.name.clone();
                let label = conn_def
                    .frontend
                    .connection_form
                    .form_data
                    .iter()
                    .find(|f| f.name == name);

                label.map(|label| Step {
                    question: format!(
                        "Please enter the `{}` for `{}`: ",
                        label.label, conn_def.name
                    ),
                    key: name.clone(),
                })
            })
            .collect::<Vec<Step>>()
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}
