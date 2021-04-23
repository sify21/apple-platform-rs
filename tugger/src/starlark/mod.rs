// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/*!
The `starlark` module and related sub-modules define the
[Starlark](https://github.com/bazelbuild/starlark) dialect used by
Tugger.
*/

pub mod code_signing;
pub mod file_resource;
pub mod macos_application_bundle_builder;
pub mod snapcraft;
#[cfg(test)]
mod testutil;
pub mod wix_bundle_builder;
pub mod wix_installer;
pub mod wix_msi_builder;

use {
    starlark::{
        environment::{Environment, EnvironmentError, TypeValues},
        values::{
            error::{RuntimeError, ValueError},
            Mutable, TypedValue, Value, ValueResult,
        },
    },
    std::ops::{Deref, DerefMut},
};

/// Holds global context for Tugger Starlark evaluation.
pub struct TuggerContext {
    pub logger: slog::Logger,
    pub code_signers: Vec<Value>,
}

impl TuggerContext {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            logger,
            code_signers: vec![],
        }
    }
}

pub struct TuggerContextValue {
    inner: TuggerContext,
}

impl Deref for TuggerContextValue {
    type Target = TuggerContext;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TuggerContextValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl TypedValue for TuggerContextValue {
    type Holder = Mutable<TuggerContextValue>;
    const TYPE: &'static str = "TuggerContext";

    fn values_for_descendant_check_and_freeze(&self) -> Box<dyn Iterator<Item = Value>> {
        Box::new(self.code_signers.clone().into_iter())
    }
}

#[derive(Default)]
pub struct TuggerContextHolder {}

impl TypedValue for TuggerContextHolder {
    type Holder = Mutable<TuggerContextHolder>;
    const TYPE: &'static str = "Tugger";

    fn values_for_descendant_check_and_freeze(&self) -> Box<dyn Iterator<Item = Value>> {
        Box::new(std::iter::empty())
    }
}

const ENVIRONMENT_CONTEXT_SYMBOL: &str = "TUGGER_CONTEXT";

pub fn get_context_value(type_values: &TypeValues) -> ValueResult {
    type_values
        .get_type_value(
            &Value::new(TuggerContextHolder::default()),
            ENVIRONMENT_CONTEXT_SYMBOL,
        )
        .ok_or_else(|| {
            ValueError::from(RuntimeError {
                code: "TUGGER",
                message: "unable to resolve context (this should never happen)".to_string(),
                label: "".to_string(),
            })
        })
}

/// Registers Tugger's Starlark dialect.
pub fn register_starlark_dialect(
    env: &mut Environment,
    type_values: &mut TypeValues,
) -> Result<(), EnvironmentError> {
    code_signing::code_signing_module(env, type_values);
    file_resource::file_resource_module(env, type_values);
    macos_application_bundle_builder::macos_application_bundle_builder_module(env, type_values);
    snapcraft::snapcraft_module(env, type_values);
    wix_bundle_builder::wix_bundle_builder_module(env, type_values);
    wix_installer::wix_installer_module(env, type_values);
    wix_msi_builder::wix_msi_builder_module(env, type_values);

    Ok(())
}

/// Populate a Starlark environment with variables needed to support this dialect.
pub fn populate_environment(
    env: &mut Environment,
    type_values: &mut TypeValues,
    context: TuggerContext,
) -> Result<(), EnvironmentError> {
    env.set(
        ENVIRONMENT_CONTEXT_SYMBOL,
        Value::new(TuggerContextValue { inner: context }),
    )?;

    let symbol = &ENVIRONMENT_CONTEXT_SYMBOL;
    type_values.add_type_value(TuggerContextHolder::TYPE, symbol, env.get(symbol)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use {super::*, crate::starlark::testutil::*, anyhow::Result};

    #[test]
    fn test_get_context() -> Result<()> {
        let env = StarlarkEnvironment::new()?;

        let context_value = get_context_value(&env.type_values).unwrap();
        context_value.downcast_ref::<TuggerContextValue>().unwrap();

        Ok(())
    }
}
