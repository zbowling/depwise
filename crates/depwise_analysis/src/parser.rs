use crate::error::AnalysisError;
use rustpython_parser::{ast, Parse};
use tracing::{debug, instrument, trace};

/// Represents a Python import statement
#[derive(Debug, Clone)]
pub struct PythonImport {
    // The module name is None for relative imports
    pub module_name: Option<String>,
    pub imported_names: Vec<String>,
    pub is_from_import: bool,
    pub is_relative: bool,
    pub alias: Option<String>,
    pub line_number: usize,
    pub relative_level: usize,
    /// Whether this import is at the top level of the module (not inside any function/class)
    pub is_top_level_import: bool,
    /// Whether this import is likely guarded by a try/except block that catches ImportError
    pub is_likely_exception_guarded: bool,
}

/// Parser for Python source code
pub struct PythonParser {
    source: String,
    nesting_level: usize,
    in_try_block: bool,
    has_import_error_handler: bool,
}

impl PythonParser {
    /// Create a new Python parser from source code
    pub fn new(source: &str) -> Self {
        debug!(
            "Creating new PythonParser with source length: {}",
            source.len()
        );
        Self {
            source: source.to_string(),
            nesting_level: 0,
            in_try_block: false,
            has_import_error_handler: false,
        }
    }

    /// Calculate line number from source position
    fn get_line_number(&self, pos: usize) -> usize {
        self.source[..pos].chars().filter(|&c| c == '\n').count() + 1
    }

    /// Process a single statement and collect any imports
    fn process_statement(&mut self, stmt: &ast::Stmt, imports: &mut Vec<PythonImport>) {
        match stmt {
            ast::Stmt::Import(import) => {
                for name in &import.names {
                    imports.push(PythonImport {
                        module_name: Some(name.name.to_string()),
                        imported_names: vec![],
                        is_from_import: false,
                        is_relative: false,
                        alias: name.asname.as_ref().map(|n| n.to_string()),
                        line_number: self.get_line_number(import.range.start().into()),
                        relative_level: 0,
                        is_top_level_import: self.nesting_level == 0,
                        is_likely_exception_guarded: self.in_try_block && self.has_import_error_handler,
                    });
                }
            }
            ast::Stmt::ImportFrom(import_from) => {
                let level = import_from.level.unwrap_or(ast::Int::new(0)).to_u32() as usize;
                let is_relative = level > 0;

                let module_name = if let Some(module) = &import_from.module {
                    Some(module.to_string())
                } else {
                    None
                };

                let mut imported_names = Vec::new();
                for name in &import_from.names {
                    if name.name.to_string() == "*" {
                        imported_names.push("*".to_string());
                    } else {
                        imported_names.push(name.name.to_string());
                    }
                }

                imports.push(PythonImport {
                    module_name,
                    imported_names,
                    is_from_import: true,
                    is_relative,
                    alias: None, // Aliases are handled per imported name
                    line_number: self.get_line_number(import_from.range.start().into()),
                    relative_level: level,
                    is_top_level_import: self.nesting_level == 0,
                    is_likely_exception_guarded: self.in_try_block && self.has_import_error_handler,
                });
            }
            // Recursively process statements in other contexts
            ast::Stmt::FunctionDef(func) => {
                self.nesting_level += 1;
                for stmt in &func.body {
                    self.process_statement(stmt, imports);
                }
                self.nesting_level -= 1;
            }
            ast::Stmt::AsyncFunctionDef(func) => {
                self.nesting_level += 1;
                for stmt in &func.body {
                    self.process_statement(stmt, imports);
                }
                self.nesting_level -= 1;
            }
            ast::Stmt::ClassDef(class) => {
                self.nesting_level += 1;
                for stmt in &class.body {
                    self.process_statement(stmt, imports);
                }
                self.nesting_level -= 1;
            }
            ast::Stmt::If(if_stmt) => {
                self.nesting_level += 1;
                for stmt in &if_stmt.body {
                    self.process_statement(stmt, imports);
                }
                for stmt in &if_stmt.orelse {
                    self.process_statement(stmt, imports);
                }
                self.nesting_level -= 1;
            }
            ast::Stmt::While(while_stmt) => {
                self.nesting_level += 1;
                for stmt in &while_stmt.body {
                    self.process_statement(stmt, imports);
                }
                self.nesting_level -= 1;
            }
            ast::Stmt::For(for_stmt) => {
                self.nesting_level += 1;
                for stmt in &for_stmt.body {
                    self.process_statement(stmt, imports);
                }
                self.nesting_level -= 1;
            }
            ast::Stmt::AsyncFor(for_stmt) => {
                self.nesting_level += 1;
                for stmt in &for_stmt.body {
                    self.process_statement(stmt, imports);
                }
                self.nesting_level -= 1;
            }
            ast::Stmt::Try(try_stmt) => {
                self.in_try_block = true;
                self.has_import_error_handler = false;

                // Check for ImportError handlers before processing the try block
                for handler in &try_stmt.handlers {
                    if let Some(except_handler) = handler.as_except_handler() {
                        // Check if this handler catches ImportError or is a catch-all
                        if let Some(exception_type) = &except_handler.type_ {
                            if let ast::Expr::Name(name) = exception_type.as_ref() {
                                let exception_name = name.id.to_string();
                                if exception_name == "ImportError"
                                    || exception_name == "Exception"
                                    || exception_name == "BaseException"
                                    || exception_name == "ModuleNotFoundError" {
                                    self.has_import_error_handler = true;
                                    break;
                                }
                            }
                        } else {
                            // No exception type specified means it's a catch-all
                            self.has_import_error_handler = true;
                            break;
                        }
                    }
                }

                // Now process the try block body with the correct handler state
                for stmt in &try_stmt.body {
                    self.process_statement(stmt, imports);
                }

                // Process the handlers
                for handler in &try_stmt.handlers {
                    if let Some(except_handler) = handler.as_except_handler() {
                        for stmt in &except_handler.body {
                            self.process_statement(stmt, imports);
                        }
                    }
                }

                for stmt in &try_stmt.orelse {
                    self.process_statement(stmt, imports);
                }

                for stmt in &try_stmt.finalbody {
                    self.process_statement(stmt, imports);
                }

                self.in_try_block = false;
                self.has_import_error_handler = false;
            }
            _ => {
                trace!("Skipping statement: {:?}", stmt);
            }
        }
    }

    /// Parse all import statements in the source code
    #[instrument(skip(self), level = "debug")]
    pub fn parse_imports(&mut self) -> Result<Vec<PythonImport>, AnalysisError> {
        let mut imports = Vec::new();

        // Parse the Python source into an AST
        let suite = ast::Suite::parse(&self.source, "<string>").map_err(|e| {
            AnalysisError::ParseFileError(
                format!("Failed to parse Python source: {}", e),
                "".to_string(),
                "".to_string(),
            )
        })?;

        // Process each statement in the AST
        for stmt in suite {
            self.process_statement(&stmt, &mut imports);
        }

        debug!(
            total_imports = imports.len(),
            "Finished parsing all imports"
        );
        Ok(imports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn init_tracing() {
        //let _ = tracing_subscriber::fmt()
        //    .with_env_filter("debug")
        //    .with_span_events(FmtSpan::CLOSE)
        //    .try_init();
    }

    #[test]
    fn test_parse_simple_imports() -> Result<(), AnalysisError> {
        init_tracing();
        let source = r#"
import os
import sys as system
from datetime import datetime
from typing import List, Dict as Dictionary
"#;

        let mut parser = PythonParser::new(source);
        let imports = parser.parse_imports()?;

        assert_eq!(imports.len(), 4);

        // Check first import
        assert_eq!(imports[0].module_name, Some("os".to_string()));
        assert!(!imports[0].is_from_import);
        assert!(!imports[0].is_relative);
        assert!(imports[0].imported_names.is_empty());
        assert_eq!(imports[0].line_number, 2);
        assert_eq!(imports[0].relative_level, 0);
        // Check second import
        assert_eq!(imports[1].module_name, Some("sys".to_string()));
        assert!(!imports[1].is_from_import);
        assert!(!imports[1].is_relative);
        assert!(imports[1].imported_names.is_empty());
        assert_eq!(imports[1].alias, Some("system".to_string()));
        assert_eq!(imports[1].relative_level, 0);

        // Check third import
        assert_eq!(imports[2].module_name, Some("datetime".to_string()));
        assert!(imports[2].is_from_import);
        assert!(!imports[2].is_relative);
        assert_eq!(imports[2].imported_names, vec!["datetime".to_string()]);

        // Check fourth import
        assert_eq!(imports[3].module_name, Some("typing".to_string()));
        assert!(imports[3].is_from_import);
        assert!(!imports[3].is_relative);
        assert_eq!(
            imports[3].imported_names,
            vec!["List".to_string(), "Dict".to_string()]
        );

        Ok(())
    }

    #[test]
    fn test_parse_dotted_as_names() -> Result<(), AnalysisError> {
        init_tracing();
        let source = r#"
import a.b.c as abc, x.y.z as xyz
import one.two, three.four as tf
"#;

        let mut parser = PythonParser::new(source);
        let imports = parser.parse_imports()?;

        assert_eq!(imports.len(), 4);

        // Check first import
        assert_eq!(imports[0].module_name, Some("a.b.c".to_string()));
        assert!(!imports[0].is_from_import);
        assert!(!imports[0].is_relative);
        assert_eq!(imports[0].alias, Some("abc".to_string()));

        // Check second import
        assert_eq!(imports[1].module_name, Some("x.y.z".to_string()));
        assert!(!imports[1].is_from_import);
        assert!(!imports[1].is_relative);
        assert_eq!(imports[1].alias, Some("xyz".to_string()));

        // Check third import
        assert_eq!(imports[2].module_name, Some("one.two".to_string()));
        assert!(!imports[2].is_from_import);
        assert!(!imports[2].is_relative);
        assert_eq!(imports[2].alias, None);

        // Check fourth import
        assert_eq!(imports[3].module_name, Some("three.four".to_string()));
        assert!(!imports[3].is_from_import);
        assert!(!imports[3].is_relative);
        assert_eq!(imports[3].alias, Some("tf".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_import_from_targets() -> Result<(), AnalysisError> {
        init_tracing();
        let source = r#"
from module import name1, name2 as alias2
from module import (name1, name2 as alias2)
from module import (name1, name2 as alias2,)
from module import *
"#;

        let mut parser = PythonParser::new(source);
        let imports = parser.parse_imports()?;

        assert_eq!(imports.len(), 4);

        // Check first import (comma-separated without parentheses)
        assert_eq!(imports[0].module_name, Some("module".to_string()));
        assert!(imports[0].is_from_import);
        assert!(!imports[0].is_relative);
        assert_eq!(
            imports[0].imported_names,
            vec!["name1".to_string(), "name2".to_string()]
        );

        // Check second import (parenthesized without trailing comma)
        assert_eq!(imports[1].module_name, Some("module".to_string()));
        assert!(imports[1].is_from_import);
        assert!(!imports[1].is_relative);
        assert_eq!(
            imports[1].imported_names,
            vec!["name1".to_string(), "name2".to_string()]
        );

        // Check third import (parenthesized with trailing comma)
        assert_eq!(imports[2].module_name, Some("module".to_string()));
        assert!(imports[2].is_from_import);
        assert!(!imports[2].is_relative);
        assert_eq!(
            imports[2].imported_names,
            vec!["name1".to_string(), "name2".to_string()]
        );

        // Check fourth import (star import)
        assert_eq!(imports[3].module_name, Some("module".to_string()));
        assert!(imports[3].is_from_import);
        assert!(!imports[3].is_relative);
        assert_eq!(imports[3].imported_names, vec!["*".to_string()]);

        Ok(())
    }

    #[test]
    fn test_parse_relative_imports() -> Result<(), AnalysisError> {
        init_tracing();
        let source = r#"
from . import name
from .. import name
from ... import name
from .module import name
from ..module import name
from ...module import name
"#;

        let mut parser = PythonParser::new(source);
        let imports = parser.parse_imports()?;

        assert_eq!(imports.len(), 6);

        // Check single dot relative import
        assert_eq!(imports[0].module_name, None);
        assert!(imports[0].is_from_import);
        assert!(imports[0].is_relative);
        assert_eq!(imports[0].imported_names, vec!["name".to_string()]);
        assert_eq!(imports[0].relative_level, 1);
        // Check double dot relative import
        assert_eq!(imports[1].module_name, None);
        assert!(imports[1].is_from_import);
        assert!(imports[1].is_relative);
        assert_eq!(imports[1].imported_names, vec!["name".to_string()]);
        assert_eq!(imports[1].relative_level, 2);

        // Check triple dot relative import
        assert_eq!(imports[2].module_name, None);
        assert!(imports[2].is_from_import);
        assert!(imports[2].is_relative);
        assert_eq!(imports[2].imported_names, vec!["name".to_string()]);
        assert_eq!(imports[2].relative_level, 3);

        // Check single dot with module
        assert_eq!(imports[3].module_name, Some("module".to_string()));
        assert!(imports[3].is_from_import);
        assert!(imports[3].is_relative);
        assert_eq!(imports[3].imported_names, vec!["name".to_string()]);
        assert_eq!(imports[3].relative_level, 1);
        // Check double dot with module
        assert_eq!(imports[4].module_name, Some("module".to_string()));
        assert!(imports[4].is_from_import);
        assert!(imports[4].is_relative);
        assert_eq!(imports[4].imported_names, vec!["name".to_string()]);
        assert_eq!(imports[4].relative_level, 2);

        // Check triple dot with module
        assert_eq!(imports[5].module_name, Some("module".to_string()));
        assert!(imports[5].is_from_import);
        assert!(imports[5].is_relative);
        assert_eq!(imports[5].imported_names, vec!["name".to_string()]);
        assert_eq!(imports[5].relative_level, 3);

        Ok(())
    }

    #[test]
    fn test_parse_complex_imports() -> Result<(), AnalysisError> {
        init_tracing();
        let source = r#"
from package.subpackage import (
    Class1,
    Class2 as Alias2,
    Class3
)
from another.package import *
import very.long.module.name as short_name

def main():
    try:
        import torch
    except ImportError:
        print("torch not found")

    print("Hello, World!")

from . import something_else

from .. import hello, world, ok

from ...hello import something_else

class Class1:
    def __init__(self):
        import os

class Class2:
    def ok(self):
        import sys

class Class3:
    def __init__(self):
        import ok
"#;

        let mut parser = PythonParser::new(source);
        let imports = parser.parse_imports()?;

        // Check first import (parenthesized)
        assert_eq!(
            imports[0].module_name,
            Some("package.subpackage".to_string())
        );
        assert!(imports[0].is_from_import);
        assert!(!imports[0].is_relative);
        assert_eq!(
            imports[0].imported_names,
            vec![
                "Class1".to_string(),
                "Class2".to_string(),
                "Class3".to_string()
            ]
        );
        assert_eq!(imports[0].line_number, 2);
        assert_eq!(imports[0].relative_level, 0);
        assert!(imports[0].is_top_level_import);
        assert!(!imports[0].is_likely_exception_guarded);

        // Check second import (star import)
        assert_eq!(imports[1].module_name, Some("another.package".to_string()));
        assert!(imports[1].is_from_import);
        assert!(!imports[1].is_relative);
        assert_eq!(imports[1].imported_names, vec!["*".to_string()]);
        assert_eq!(imports[1].line_number, 7);
        assert!(imports[1].is_top_level_import);
        assert!(!imports[1].is_likely_exception_guarded);

        // Check third import (with alias)
        assert_eq!(
            imports[2].module_name,
            Some("very.long.module.name".to_string())
        );
        assert!(!imports[2].is_from_import);
        assert!(!imports[2].is_relative);
        assert_eq!(imports[2].alias, Some("short_name".to_string()));
        assert_eq!(imports[2].line_number, 8);
        assert!(imports[2].is_top_level_import);
        assert!(!imports[2].is_likely_exception_guarded);

        // Check fourth import (inside try block)
        assert_eq!(imports[3].module_name, Some("torch".to_string()));
        assert!(!imports[3].is_from_import);
        assert!(!imports[3].is_relative);
        assert_eq!(imports[3].alias, None);
        assert_eq!(imports[3].line_number, 12);
        assert!(!imports[3].is_top_level_import);
        assert!(imports[3].is_likely_exception_guarded);

        // Check fifth import (relative import with single dot)
        assert_eq!(imports[4].module_name, None);
        assert!(imports[4].is_from_import);
        assert!(imports[4].is_relative);
        assert_eq!(
            imports[4].imported_names,
            vec!["something_else".to_string()]
        );
        assert_eq!(imports[4].line_number, 18);
        assert_eq!(imports[4].relative_level, 1);
        assert!(imports[4].is_top_level_import);
        assert!(!imports[4].is_likely_exception_guarded);

        // Check sixth import (relative import with two dots)
        assert_eq!(imports[5].module_name, None);
        assert!(imports[5].is_from_import);
        assert!(imports[5].is_relative);
        assert_eq!(
            imports[5].imported_names,
            vec!["hello".to_string(), "world".to_string(), "ok".to_string()]
        );
        assert_eq!(imports[5].line_number, 20);
        assert_eq!(imports[5].relative_level, 2);
        assert!(imports[5].is_top_level_import);
        assert!(!imports[5].is_likely_exception_guarded);

        // Check seventh import (relative import with three dots)
        assert_eq!(imports[6].module_name, Some("hello".to_string()));
        assert!(imports[6].is_from_import);
        assert!(imports[6].is_relative);
        assert_eq!(
            imports[6].imported_names,
            vec!["something_else".to_string()]
        );
        assert_eq!(imports[6].line_number, 22);
        assert_eq!(imports[6].relative_level, 3);
        assert!(imports[6].is_top_level_import);
        assert!(!imports[6].is_likely_exception_guarded);

        // Check eighth import (inside class)
        assert_eq!(imports[7].module_name, Some("os".to_string()));
        assert!(!imports[7].is_from_import);
        assert!(!imports[7].is_relative);
        assert!(imports[7].imported_names.is_empty());
        assert_eq!(imports[7].line_number, 26);
        assert!(!imports[7].is_top_level_import);
        assert!(!imports[7].is_likely_exception_guarded);

        // Check ninth import (inside class)
        assert_eq!(imports[8].module_name, Some("sys".to_string()));
        assert!(!imports[8].is_from_import);
        assert!(!imports[8].is_relative);
        assert!(imports[8].imported_names.is_empty());
        assert_eq!(imports[8].line_number, 30);
        assert!(!imports[8].is_top_level_import);
        assert!(!imports[8].is_likely_exception_guarded);

        // Check tenth import (inside class)
        assert_eq!(imports[9].module_name, Some("ok".to_string()));
        assert!(!imports[9].is_from_import);
        assert!(!imports[9].is_relative);
        assert!(imports[9].imported_names.is_empty());
        assert_eq!(imports[9].line_number, 34);
        assert!(!imports[9].is_top_level_import);
        assert!(!imports[9].is_likely_exception_guarded);

        Ok(())
    }

    #[test]
    fn test_parse_guarded_imports() -> Result<(), AnalysisError> {
        init_tracing();
        let source = r#"
try:
    import optional_package
except ImportError:
    print("optional_package not found")

try:
    import another_optional
except Exception:
    print("another_optional not found")

try:
    import third_optional
except:
    print("third_optional not found")

def function():
    try:
        import nested_optional
    except ImportError:
        print("nested_optional not found")
"#;

        let mut parser = PythonParser::new(source);
        let imports = parser.parse_imports()?;

        assert_eq!(imports.len(), 4);

        // Check first import (with ImportError handler)
        assert_eq!(imports[0].module_name, Some("optional_package".to_string()));
        assert!(imports[0].is_likely_exception_guarded);
        assert!(imports[0].is_top_level_import);

        // Check second import (with generic Exception handler)
        assert_eq!(imports[1].module_name, Some("another_optional".to_string()));
        assert!(imports[1].is_likely_exception_guarded);
        assert!(imports[1].is_top_level_import);

        // Check third import (with catch-all handler)
        assert_eq!(imports[2].module_name, Some("third_optional".to_string()));
        assert!(imports[2].is_likely_exception_guarded);
        assert!(imports[2].is_top_level_import);

        // Check fourth import (nested in function with ImportError handler)
        assert_eq!(imports[3].module_name, Some("nested_optional".to_string()));
        assert!(imports[3].is_likely_exception_guarded);
        assert!(!imports[3].is_top_level_import);

        Ok(())
    }
}
