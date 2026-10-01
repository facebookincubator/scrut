/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Procedural macros for scrutty
//!
//! Provides attribute macros for declarative test registration.
//! The scenario name is always derived from the source file name.
//!
//! # Example
//!
//! ```rust,ignore
//! use scrutty::prelude::*;
//!
//! // Register tests - scenario name is derived from filename (e.g., sync.rs -> "sync")
//! #[scenario_test]
//! fn test_basic_functionality() -> TestResult {
//!     Ok(())
//! }
//!
//! // With named resource - the constant is injected into the function
//! #[scenario_test(resource MAST_JOB = "mast:gtn_infra")]
//! fn test_with_resource() -> TestResult {
//!     // MAST_JOB constant is available here
//!     let job: MastJobHandle = resource_handle(MAST_JOB)?;
//!     Ok(())
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::Expr;
use syn::ExprArray;
use syn::Ident;
use syn::ItemFn;
use syn::LitStr;
use syn::Token;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::spanned::Spanned;

/// A named resource binding: CONST_NAME = "resource_id"
struct NamedResource {
    name: Ident,
    value: String,
}

/// Arguments for the `#[scenario_test]` attribute
struct ScenarioTestArgs {
    /// Named resources (injected as constants into the function)
    named_resources: Vec<NamedResource>,
}

impl ScenarioTestArgs {
    fn resource_values(&self) -> Vec<&str> {
        self.named_resources
            .iter()
            .map(|r| r.value.as_str())
            .collect()
    }
}

impl Parse for ScenarioTestArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut named_resources = Vec::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            if ident == "resource" {
                // Parse: resource CONST_NAME = "value"
                let const_name: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                named_resources.push(NamedResource {
                    name: const_name,
                    value: value.value(),
                });
            } else if ident == "resources" {
                // Legacy syntax: resources = ["id1", "id2"]
                // Convert to named resources with generated names (RESOURCE_0, RESOURCE_1, etc.)
                input.parse::<Token![=]>()?;
                let array: ExprArray = input.parse()?;
                for (i, elem) in array.elems.iter().enumerate() {
                    if let Expr::Lit(lit) = elem {
                        if let syn::Lit::Str(s) = &lit.lit {
                            let const_name = Ident::new(&format!("RESOURCE_{}", i), elem.span());
                            named_resources.push(NamedResource {
                                name: const_name,
                                value: s.value(),
                            });
                        } else {
                            return Err(syn::Error::new_spanned(lit, "expected string literal"));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(elem, "expected string literal"));
                    }
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unknown attribute: {}", ident),
                ));
            }

            // Optional trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ScenarioTestArgs { named_resources })
    }
}

/// Attribute macro for registering scenario tests.
///
/// This macro registers a test function with the global test registry,
/// allowing automatic collection without manual `scenario()` functions.
/// The scenario name is always derived from the source file name.
///
/// # Arguments
///
/// * `resource` - A named resource binding: `resource CONST_NAME = "resource_id"` (can be repeated)
///
/// # Example
///
/// ```rust,ignore
/// use scrutty::prelude::*;
///
/// // Scenario derived from filename (e.g., sync.rs -> "sync")
/// #[scenario_test]
/// fn test_basic_functionality() -> TestResult {
///     Ok(())
/// }
///
/// // With named resource - constant is injected into function body
/// #[scenario_test(resource MAST_JOB = "mast:gtn_infra")]
/// fn test_with_mast_job() -> TestResult {
///     let job: MastJobHandle = resource_handle(MAST_JOB)?;
///     Ok(())
/// }
///
/// // Multiple named resources
/// #[scenario_test(
///     resource MAST_JOB = "mast:gtn_infra",
///     resource DATABASE = "db:test"
/// )]
/// fn test_with_multiple_resources() -> TestResult {
///     let job: MastJobHandle = resource_handle(MAST_JOB)?;
///     let db = resource_handle::<Database>(DATABASE)?;
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn scenario_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ScenarioTestArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_attrs = &input.attrs;
    let fn_vis = &input.vis;
    let fn_sig = &input.sig;
    let fn_block = &input.block;

    // Generate const declarations for named resources
    let resource_consts: Vec<_> = args
        .named_resources
        .iter()
        .map(|r| {
            let name = &r.name;
            let value = &r.value;
            quote! {
                const #name: &str = #value;
            }
        })
        .collect();

    // Get resource values for registration
    let resource_values: Vec<_> = args
        .resource_values()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Create the modified function with injected constants
    let modified_fn = if resource_consts.is_empty() {
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig #fn_block
        }
    } else {
        let stmts = &fn_block.stmts;
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig {
                #(#resource_consts)*
                #(#stmts)*
            }
        }
    };

    // Derive scenario from filename using file!()
    // Use RegisteredTestFromFile for const-compatible file-based registration
    let expanded = quote! {
        #modified_fn

        ::scrutty::inventory::submit! {
            ::scrutty::RegisteredTestFromFile::new(
                file!(),
                #fn_name_str,
                #fn_name,
                &[#(#resource_values),*],
            )
        }
    };

    TokenStream::from(expanded)
}
