use std::io::{Result, Error, ErrorKind};

use handlebars::Handlebars;
use log::error;

use crate::types::ExecResult;

pub(crate) trait Output {
    fn print(data: &Vec<ExecResult>) -> Result<String>;
}

pub(crate) struct HTML;

impl Output for HTML {
    fn print(data: &Vec<ExecResult>) -> Result<String> {
        // To generate htlm output, I have to use templates because I haven't found any other good
        // solution
        let template = r#"
<table>
    <tr>
        <th>Chart Name</th>
        <th>Current Version</th>
        <th>Latest Version</th>
        <th>Status</th>
    </tr>
    {{#each this as |tr|}}
    <tr>
        <th>{{tr.name}}</th>
        <th>{{tr.current_version}}</th>
        <th>{{tr.latest_version}}</th>
        <th>{{tr.status}}</th>
    </tr>
    {{/each}}
</table>
"#;

        let mut reg = Handlebars::new();
        // TODO: Handle this error
        reg.register_template_string("html_table", template)
            .unwrap();

        match reg.render("html_table", &data) {
            Ok(res) => Ok(res),
            Err(err) => {
                error!("{}", err);
                return Err(Error::new(ErrorKind::InvalidInput, err.to_string()));
            }
        }
    }
}

pub(crate) struct YAML;

impl Output for YAML {
    fn print(data: &Vec<ExecResult>) -> Result<String> {
        match serde_yaml::to_string(&data) {
            Ok(res) => return Ok(res),
            Err(err) => return Err(Error::new(ErrorKind::InvalidData, err.to_string())),
        }
    }
}
