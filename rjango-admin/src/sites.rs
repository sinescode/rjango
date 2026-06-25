use std::collections::HashMap;
use rjango_core::{Request, Response};

/// The admin site — holds all registered ModelAdmin instances and renders CRUD.
pub struct AdminSite {
    pub site_title: String,
    pub site_header: String,
    pub index_title: String,
    pub site_url: String,
    registrations: HashMap<String, HashMap<String, crate::ModelAdmin>>, // app_label -> {model_name -> admin}
}

impl AdminSite {
    pub fn new() -> Self {
        Self {
            site_title: "Rjango Admin".into(),
            site_header: "Rjango Administration".into(),
            index_title: "Site Administration".into(),
            site_url: "/admin/".into(),
            registrations: HashMap::new(),
        }
    }

    pub fn register(&mut self, app_label: &str, admin: crate::ModelAdmin) {
        self.registrations
            .entry(app_label.to_string())
            .or_default()
            .insert(admin.model_name.clone(), admin);
    }

    pub fn unregister(&mut self, app_label: &str, model_name: &str) {
        if let Some(app) = self.registrations.get_mut(app_label) {
            app.remove(model_name);
        }
    }

    pub fn is_registered(&self, app_label: &str, model_name: &str) -> bool {
        self.registrations
            .get(app_label)
            .and_then(|app| app.get(model_name))
            .is_some()
    }

    /// Get all registered apps.
    pub fn get_registered_apps(&self) -> Vec<String> {
        self.registrations.keys().cloned().collect()
    }

    /// Get all models for an app.
    pub fn get_models(&self, app_label: &str) -> Vec<&crate::ModelAdmin> {
        self.registrations
            .get(app_label)
            .map(|m| m.values().collect())
            .unwrap_or_default()
    }

    /// Dispatch an admin URL to the appropriate handler.
    pub fn dispatch(&self, request: &Request, path_segments: &[&str]) -> Response {
        // /admin/ -> index
        // /admin/<app_label>/ -> app index
        // /admin/<app_label>/<model_name>/ -> list
        // /admin/<app_label>/<model_name>/<id>/change/ -> change form
        // /admin/<app_label>/<model_name>/<id>/delete/ -> delete confirm
        // /admin/<app_label>/<model_name>/add/ -> add form

        match path_segments.len() {
            0 => self.index(request),
            1 => {
                let app_label = path_segments[0];
                self.app_index(request, app_label)
            }
            2 => {
                let app_label = path_segments[0];
                let model_name = path_segments[1];
                self.list_view(request, app_label, model_name)
            }
            3 => {
                let app_label = path_segments[0];
                let model_name = path_segments[1];
                let action = path_segments[2];
                match action {
                    "add" => self.add_view(request, app_label, model_name),
                    _ => Response::not_found(),
                }
            }
            4 => {
                let app_label = path_segments[0];
                let model_name = path_segments[1];
                let _id = path_segments[2];
                let action = path_segments[3];
                match action {
                    "change" => self.change_view(request, app_label, model_name),
                    "delete" => self.delete_view(request, app_label, model_name),
                    _ => Response::not_found(),
                }
            }
            _ => Response::not_found(),
        }
    }

    /// Render the admin index page.
    pub fn index(&self, _request: &Request) -> Response {
        let mut body = String::from(
            r#"<!DOCTYPE html><html lang="en"><head>
            <meta charset="UTF-8">
            <title>Rjango Admin</title>
            <style>
                body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;
                     margin:0;background:#f5f5f5;color:#333}
                .header{background:#417690;color:#fff;padding:10px 20px}
                .header h1{margin:0;font-size:20px}
                .header a{color:#fff;text-decoration:none}
                .container{max-width:960px;margin:20px auto;padding:0 20px}
                .card{background:#fff;border-radius:4px;box-shadow:0 1px 3px rgba(0,0,0,0.1);margin-bottom:20px}
                .card-header{background:#f8f8f8;padding:10px 15px;border-bottom:1px solid #eee;font-weight:600}
                .card-body{padding:0}
                .model-list{list-style:none;margin:0;padding:0}
                .model-list li{border-bottom:1px solid #f0f0f0}
                .model-list li:last-child{border-bottom:none}
                .model-list a{display:block;padding:10px 15px;color:#447e9b;text-decoration:none}
                .model-list a:hover{background:#f0f8ff}
            </style></head><body>
            <div class="header"><h1><a href="/admin/">Rjango Administration</a></h1></div>
            <div class="container"><h2>Site Administration</h2>"#
        );

        if self.registrations.is_empty() {
            body.push_str("<div class=\"card\"><div class=\"card-body\"><p style=\"padding:15px;margin:0;color:#999\">No models registered.</p></div></div>");
        } else {
            let mut apps: Vec<&String> = self.registrations.keys().collect();
            apps.sort();
            for app_label in apps {
                let models = self.get_models(app_label);
                body.push_str(&format!(
                    "<div class=\"card\"><div class=\"card-header\">{}</div><div class=\"card-body\"><ul class=\"model-list\">",
                    app_label
                ));
                for admin in models {
                    body.push_str(&format!(
                        "<li><a href=\"/admin/{}/{}/\">{}</a></li>",
                        app_label, admin.model_name.to_lowercase(), admin.model_name
                    ));
                }
                body.push_str("</ul></div></div>");
            }
        }

        body.push_str("</div></body></html>");
        Response::html(body)
    }

    fn app_index(&self, _request: &Request, app_label: &str) -> Response {
        if !self.registrations.contains_key(app_label) {
            return Response::not_found();
        }
        // Redirect to the first model's list view
        if let Some(models) = self.registrations.get(app_label) {
            if let Some(name) = models.keys().next() {
                return Response::redirect(&format!("/admin/{}/{}/", app_label, name), false);
            }
        }
        Response::not_found()
    }

    /// List view for a model.
    pub fn list_view(&self, _request: &Request, app_label: &str, model_name: &str) -> Response {
        let admin = self.registrations.get(app_label)
            .and_then(|m| m.get(model_name));
        let admin = match admin {
            Some(a) => a,
            None => return Response::not_found(),
        };

        let display_fields = &admin.list_display;
        let mut headers = String::new();
        let mut rows = String::new();
        for f in display_fields {
            headers.push_str(&format!("<th>{}</th>", f));
        }
        // Placeholder: in production, this would query the database
        let placeholder_row: String = display_fields.iter().map(|_| {
            format!("<td>&mdash;</td>")
        }).collect();
        rows.push_str(&format!("<tr>{}</tr>", placeholder_row));

        let body = format!(
            r#"<!DOCTYPE html><html lang="en"><head>
            <meta charset="UTF-8"><title>{} – Rjango Admin</title>
            <style>
                body{{font-family:-apple-system,sans-serif;margin:0;background:#f5f5f5;color:#333}}
                .header{{background:#417690;color:#fff;padding:10px 20px}}
                .header h1{{margin:0;font-size:20px}}
                .header a{{color:#fff;text-decoration:none}}
                .container{{max-width:960px;margin:20px auto;padding:0 20px}}
                .card{{background:#fff;border-radius:4px;box-shadow:0 1px 3px rgba(0,0,0,0.1);margin-bottom:20px}}
                .card-header{{background:#f8f8f8;padding:10px 15px;border-bottom:1px solid #eee;font-weight:600}}
                .card-body{{padding:15px}}
                table{{width:100%;border-collapse:collapse}}
                th,td{{padding:8px 12px;text-align:left;border-bottom:1px solid #eee}}
                th{{background:#f8f8f8;font-weight:600}}
                tr:hover{{background:#f0f8ff}}
                .btn{{display:inline-block;padding:6px 12px;border-radius:4px;text-decoration:none;
                       font-size:13px}}
                .btn-primary{{background:#417690;color:#fff}}
                .btn-primary:hover{{background:#305f72}}
                .actions{{margin-bottom:15px}}
            </style></head><body>
            <div class="header"><h1><a href="/admin/">Rjango Admin</a></h1></div>
            <div class="container">
            <h2>{} <span style="font-weight:400;color:#999">– {}</span></h2>
            <div class="actions"><a href="add/" class="btn btn-primary">Add {}</a></div>
            <div class="card">
            <div class="card-header">{}</div>
            <div class="card-body">
            <table><thead><tr>{}</tr></thead><tbody>{}</tbody></table>
            </div></div></div></body></html>"#,
            admin.model_name,
            admin.model_name, app_label,
            admin.model_name,
            format!("{} objects", admin.model_name),
            headers, rows
        );
        Response::html(body)
    }

    pub fn add_view(&self, _request: &Request, app_label: &str, model_name: &str) -> Response {
        let admin = self.registrations.get(app_label)
            .and_then(|m| m.get(model_name));
        let admin = match admin {
            Some(a) => a,
            None => return Response::not_found(),
        };

        let body = format!(
            r#"<!DOCTYPE html><html lang="en"><head>
            <meta charset="UTF-8"><title>Add {} – Rjango Admin</title>
            <style>
                body{{font-family:-apple-system,sans-serif;margin:0;background:#f5f5f5;color:#333}}
                .header{{background:#417690;color:#fff;padding:10px 20px}}
                .header h1{{margin:0;font-size:20px}}
                .container{{max-width:600px;margin:20px auto;padding:0 20px}}
                .card{{background:#fff;border-radius:4px;box-shadow:0 1px 3px rgba(0,0,0,0.1);margin-bottom:20px}}
                .card-header{{background:#f8f8f8;padding:10px 15px;border-bottom:1px solid #eee;font-weight:600}}
                .card-body{{padding:15px}}
                label{{display:block;margin-bottom:5px;font-weight:600;font-size:13px}}
                input[type=text],input[type=number],textarea,select{{width:100%;padding:8px;
                       border:1px solid #ccc;border-radius:4px;box-sizing:border-box;margin-bottom:15px}}
                .btn{{display:inline-block;padding:8px 16px;border-radius:4px;text-decoration:none;
                       border:none;cursor:pointer;font-size:14px}}
                .btn-primary{{background:#417690;color:#fff}}
                .btn-secondary{{background:#999;color:#fff;margin-left:8px}}
            </style></head><body>
            <div class="header"><h1><a href="/admin/">Rjango Admin</a></h1></div>
            <div class="container">
            <h2>Add {}</h2>
            <div class="card">
            <div class="card-header">{}</div>
            <div class="card-body">
            <form method="post">
                <p style="color:#999;margin-bottom:15px">Form fields would appear here.</p>
                <button type="submit" class="btn btn-primary">Save</button>
                <a href="../" class="btn btn-secondary">Cancel</a>
            </form>
            </div></div></div></body></html>"#,
            admin.model_name, admin.model_name, admin.model_name
        );
        Response::html(body)
    }

    pub fn change_view(&self, _request: &Request, app_label: &str, model_name: &str) -> Response {
        let admin = self.registrations.get(app_label)
            .and_then(|m| m.get(model_name));
        let admin = match admin {
            Some(a) => a,
            None => return Response::not_found(),
        };

        let body = format!(
            r#"<!DOCTYPE html><html lang="en"><head>
            <meta charset="UTF-8"><title>Change {} – Rjango Admin</title>
            <style>
                body{{font-family:-apple-system,sans-serif;margin:0;background:#f5f5f5;color:#333}}
                .header{{background:#417690;color:#fff;padding:10px 20px}}
                .header h1{{margin:0;font-size:20px}}
                .container{{max-width:600px;margin:20px auto;padding:0 20px}}
                .card{{background:#fff;border-radius:4px;box-shadow:0 1px 3px rgba(0,0,0,0.1);margin-bottom:20px}}
                .card-header{{background:#f8f8f8;padding:10px 15px;border-bottom:1px solid #eee;font-weight:600}}
                .card-body{{padding:15px}}
                .btn{{display:inline-block;padding:8px 16px;border-radius:4px;text-decoration:none;
                       border:none;cursor:pointer;font-size:14px}}
                .btn-primary{{background:#417690;color:#fff}}
                .btn-danger{{background:#ba2121;color:#fff}}
                .btn-secondary{{background:#999;color:#fff;margin-left:8px}}
            </style></head><body>
            <div class="header"><h1><a href="/admin/">Rjango Admin</a></h1></div>
            <div class="container">
            <h2>Change {}</h2>
            <div class="card">
            <div class="card-header">{}</div>
            <div class="card-body">
            <form method="post">
                <p style="color:#999;margin-bottom:15px">Change form would appear here.</p>
                <button type="submit" class="btn btn-primary">Save</button>
                <a href="../delete/" class="btn btn-danger">Delete</a>
                <a href="../" class="btn btn-secondary">Cancel</a>
            </form>
            </div></div></div></body></html>"#,
            admin.model_name, admin.model_name, admin.model_name
        );
        Response::html(body)
    }

    pub fn delete_view(&self, _request: &Request, app_label: &str, model_name: &str) -> Response {
        let admin = self.registrations.get(app_label)
            .and_then(|m| m.get(model_name));
        let admin = match admin {
            Some(a) => a,
            None => return Response::not_found(),
        };

        let body = format!(
            r#"<!DOCTYPE html><html lang="en"><head>
            <meta charset="UTF-8"><title>Delete {} – Rjango Admin</title>
            <style>
                body{{font-family:-apple-system,sans-serif;margin:0;background:#f5f5f5;color:#333}}
                .header{{background:#417690;color:#fff;padding:10px 20px}}
                .header h1{{margin:0;font-size:20px}}
                .container{{max-width:600px;margin:20px auto;padding:0 20px}}
                .card{{background:#fff;border-radius:4px;box-shadow:0 1px 3px rgba(0,0,0,0.1);margin-bottom:20px}}
                .card-body{{padding:15px}}
                .btn{{display:inline-block;padding:8px 16px;border-radius:4px;text-decoration:none;
                       border:none;cursor:pointer;font-size:14px}}
                .btn-danger{{background:#ba2121;color:#fff}}
                .btn-secondary{{background:#999;color:#fff;margin-left:8px}}
            </style></head><body>
            <div class="header"><h1><a href="/admin/">Rjango Admin</a></h1></div>
            <div class="container">
            <h2>Delete {}</h2>
            <div class="card">
            <div class="card-body">
            <p>Are you sure you want to delete this {}?</p>
            <form method="post">
                <button type="submit" class="btn btn-danger">Yes, I'm sure</button>
                <a href="../" class="btn btn-secondary">Cancel</a>
            </form>
            </div></div></div></body></html>"#,
            admin.model_name, admin.model_name, admin.model_name
        );
        Response::html(body)
    }

    /// Get URL patterns for mounting the admin site.
    pub fn url_patterns(&self) -> Vec<(&'static str, fn(rjango_core::Request) -> rjango_core::Response, Option<&'static str>)> {
        vec![]
    }

    /// Generate a URLResolver with admin routes.
    /// This is the recommended way to mount admin URLs.
    pub fn urls(&self) -> rjango_urls::URLResolver {
        let mut patterns = vec![
            rjango_urls::URLPattern::new("/admin/", |req| {
                crate::ADMIN_SITE.lock().unwrap().index(&req)
            }, Some("admin:index")),
        ];

        for app_label in self.get_registered_apps() {
            let models = self.get_models(&app_label);
            for admin in models {
                let app_changelist = app_label.clone();
                let model_changelist = admin.model_name.clone();
                let path_changelist = format!("/admin/{}/{}/", app_changelist, model_changelist.to_lowercase());
                let name_changelist = format!("admin:{}_{}_changelist", app_changelist, model_changelist.to_lowercase());

                let app_add = app_label.clone();
                let model_add = admin.model_name.clone();
                let path_add = format!("/admin/{}/{}/add/", app_add, model_add.to_lowercase());
                let name_add = format!("admin:{}_{}_add", app_add, model_add.to_lowercase());

                patterns.push(rjango_urls::URLPattern::new(
                    &path_changelist,
                    move |req| crate::ADMIN_SITE.lock().unwrap().list_view(&req, &app_changelist, &model_changelist),
                    Some(&name_changelist),
                ));

                patterns.push(rjango_urls::URLPattern::new(
                    &path_add,
                    move |req| crate::ADMIN_SITE.lock().unwrap().add_view(&req, &app_add, &model_add),
                    Some(&name_add),
                ));
            }
        }

        rjango_urls::URLResolver::new(patterns)
    }
}
