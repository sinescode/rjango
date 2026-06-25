use crate::context::Context;
use crate::loaders::TemplateLoader;

/// Template engine — loads and renders templates.
pub struct Engine {
    pub loader: Box<dyn TemplateLoader>,
    pub dirs: Vec<std::path::PathBuf>,
}

impl Engine {
    pub fn new(loader: Box<dyn TemplateLoader>) -> Self {
        Self { loader, dirs: vec![] }
    }

    pub fn with_dirs(mut self, dirs: Vec<std::path::PathBuf>) -> Self {
        self.dirs = dirs;
        self
    }

    /// Render a template by name with the given context.
    pub fn render(&self, name: &str, ctx: &Context) -> rjango_core::Result<String> {
        let source = self.loader.load(name).ok_or_else(|| {
            rjango_core::RjangoError::Template(format!("Template not found: {}", name))
        })?;
        self.render_string(&source, ctx)
    }

    /// Render a template string with the given context.
    pub fn render_string(&self, source: &str, ctx: &Context) -> rjango_core::Result<String> {
        // Simple variable interpolation: {{ var }}
        // Supports dot access: {{ var.key }}
        let re = regex::Regex::new(r"\{\{\s*([\w.]+)\s*\}\}").unwrap();
        let result = re.replace_all(source, |caps: &regex::Captures| {
            let var_path = caps.get(1).unwrap().as_str();
            ctx.get(var_path)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default()
        });

        // Simple block tags: {% if var %}...{% endif %}
        // This is a minimal implementation.
        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_variable() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let mut ctx = Context::new();
        ctx.insert("name".into(), serde_json::Value::String("World".into()));
        let result = engine.render_string("Hello {{ name }}!", &ctx).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_render_missing_variable() {
        let engine = Engine::new(Box::new(crate::loaders::TestLoader));
        let ctx = Context::new();
        let result = engine.render_string("Hello {{ missing }}!", &ctx).unwrap();
        assert_eq!(result, "Hello !");
    }
}
