use std::ptr;
use std::{ffi::CStr, path::PathBuf};

use glib_sys::{GError, g_clear_error, g_error_free};
use gtk::glib::gobject_ffi::g_object_unref;
use tracker_sys::{
    TrackerSparqlConnection, tracker_sparql_connection_query, tracker_sparql_cursor_get_n_columns,
    tracker_sparql_cursor_get_string, tracker_sparql_cursor_next,
};

#[derive(Debug, Clone)]
pub struct FileData {
    pub mime: Option<String>,
    pub path: Option<PathBuf>,
    pub uri: String,
}

impl FileData {
    pub fn try_open(&self) -> bool {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(&self.uri);

        match cmd.spawn() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[inline]
    pub fn icon(&self) -> gtk::gio::Icon {
        let mut icon_name = "text-x-preview";
        if self.mime.is_some() {
            let mime = self.mime.clone().unwrap();
            let attempt = mime.replacen("/", "-", 1);
            icon_name = attempt.as_str();
            crate::icons::get_icon(icon_name)
        } else {
            crate::icons::get_icon(icon_name)
        }
    }
}

pub fn search<S: ToString>(term: S, limit: i64) -> Result<Vec<FileData>, String> {
    let mut results = vec![];
    let term = term.to_string();
    let query = format!(
        r#"
SELECT ?uri ?mimetype ?type ?snippet
WHERE {{
  {{
    SELECT (?s AS ?uri) ?mimetype ?type (fts:snippet(?s, "", "") AS ?snippet) {{
      GRAPH tracker:FileSystem {{
        ?s a nfo:FileDataObject ;
           fts:match "{term}" ;
           rdf:type ?type ;
           nie:dataSource ?ds .
         ?ie nie:isStoredAs ?s;
           nie:mimeType ?mimetype .
        OPTIONAL {{ ?ds tracker:available ?available }} .
        FILTER (IF (true, true, ?available)) .
      }}
    }}
  }} UNION {{
    SELECT ?uri ?mimetype ?type (fts:snippet(?s, "", "") AS ?snippet) {{
      GRAPH ?g {{
        ?s a nie:InformationElement ;
           fts:match "{term}" ;
           rdf:type ?type ;
           nie:mimeType ?mimetype ;
           nie:isStoredAs ?uri .
                 }}
                 GRAPH tracker:FileSystem {{
        ?uri nie:dataSource ?ds .
        OPTIONAL {{ ?ds tracker:available ?available }} .
        FILTER (IF (true, true, ?available)) .
      }}
    }}
  }}
}}
GROUP BY ?uri
ORDER BY ?uri
OFFSET 0
LIMIT {limit}"#
    );

    let query_str = format!("{query}\0");

    #[allow(unused_assignments)]
    let mut conn: *mut TrackerSparqlConnection = ptr::null_mut();
    let mut error: *mut GError = ptr::null_mut();

    let service_name = "org.freedesktop.Tracker3.Miner.Files\0";

    conn = unsafe {
        tracker_sys::tracker_sparql_connection_bus_new(
            service_name.as_ptr() as *const i8,
            ptr::null(),
            ptr::null_mut(),
            &mut error,
        )
    };

    if conn.is_null() {
        unsafe {
            g_clear_error(&mut error);
        }
        return Err("Could not establish a connection to Tracker".into());
    }

    let cursor = unsafe {
        tracker_sparql_connection_query(
            conn,
            query_str.as_str().as_ptr() as *const i8,
            ptr::null_mut(),
            &mut error,
        )
    };

    if !error.is_null() {
        let err_msg: String;
        unsafe {
            err_msg = format!("{}", CStr::from_ptr((*error).message).to_str().unwrap());
            g_error_free(error);
        }
        return Err(format!("Could not get search results: {}", err_msg).into());
    }

    if cursor.is_null() {
        return Err("No results were found matching your query".into());
    }

    while unsafe { tracker_sparql_cursor_next(cursor, ptr::null_mut(), &mut error) == 1 } {
        unsafe {
            let _ = tracker_sparql_cursor_get_n_columns(cursor);
            let uri = CStr::from_ptr(tracker_sparql_cursor_get_string(cursor, 0, ptr::null_mut()))
                .to_string_lossy()
                .to_string();

            let mut path = None;
            if uri.starts_with("file://") {
                path = Some(PathBuf::from(
                    url_escape::decode(uri.strip_prefix("file://").unwrap()).to_string(),
                ));
            }

            let mime_ptr = tracker_sparql_cursor_get_string(cursor, 1, ptr::null_mut());
            let mime = if mime_ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(mime_ptr).to_string_lossy().to_string())
            };

            results.push(FileData { mime, uri, path });
        }
    }

    unsafe {
        g_object_unref(conn.cast());
    }

    Ok(results)
}
