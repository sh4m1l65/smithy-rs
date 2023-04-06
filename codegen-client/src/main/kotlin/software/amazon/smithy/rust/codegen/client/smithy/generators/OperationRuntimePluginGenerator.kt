/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package software.amazon.smithy.rust.codegen.client.smithy.generators

import software.amazon.smithy.model.shapes.OperationShape
import software.amazon.smithy.rust.codegen.client.smithy.ClientCodegenContext
import software.amazon.smithy.rust.codegen.core.rustlang.RustWriter
import software.amazon.smithy.rust.codegen.core.rustlang.rustTemplate
import software.amazon.smithy.rust.codegen.core.rustlang.writable
import software.amazon.smithy.rust.codegen.core.smithy.RuntimeType
import software.amazon.smithy.rust.codegen.core.smithy.customize.NamedCustomization
import software.amazon.smithy.rust.codegen.core.smithy.customize.Section
import software.amazon.smithy.rust.codegen.core.smithy.customize.writeCustomizations

sealed class OperationRuntimePluginSection(name: String) : Section(name) {
    /**
     * Hook for adding additional things to config inside operation runtime plugins.
     */
    data class AdditionalConfig(
        val configBagName: String,
        val operationShape: OperationShape,
    ) : OperationRuntimePluginSection("AdditionalConfig")
}

typealias OperationRuntimePluginCustomization = NamedCustomization<OperationRuntimePluginSection>

/**
 * Generates operation-level runtime plugins
 */
class OperationRuntimePluginGenerator(
    codegenContext: ClientCodegenContext,
) {
    private val codegenScope = codegenContext.runtimeConfig.let { rc ->
        val runtimeApi = RuntimeType.smithyRuntimeApi(rc)
        arrayOf(
            "AuthOptionListResolverParams" to runtimeApi.resolve("client::auth::option_resolver::AuthOptionListResolverParams"),
            "AuthOptionResolverParams" to runtimeApi.resolve("client::orchestrator::AuthOptionResolverParams"),
            "BoxError" to runtimeApi.resolve("client::runtime_plugin::BoxError"),
            "ConfigBag" to runtimeApi.resolve("config_bag::ConfigBag"),
            "ConfigBagAccessors" to runtimeApi.resolve("client::orchestrator::ConfigBagAccessors"),
            "RuntimePlugin" to runtimeApi.resolve("client::runtime_plugin::RuntimePlugin"),
        )
    }

    fun render(
        writer: RustWriter,
        operationShape: OperationShape,
        operationStructName: String,
        customizations: List<OperationRuntimePluginCustomization>,
    ) {
        writer.rustTemplate(
            """
            impl #{RuntimePlugin} for $operationStructName {
                fn configure(&self, cfg: &mut #{ConfigBag}) -> Result<(), #{BoxError}> {
                    use #{ConfigBagAccessors} as _;
                    cfg.set_request_serializer(${operationStructName}RequestSerializer);
                    cfg.set_response_deserializer(${operationStructName}ResponseDeserializer);

                    ${"" /* TODO(IdentityAndAuth): Resolve auth parameters from input for services that need this */}
                    cfg.set_auth_option_resolver_params(#{AuthOptionResolverParams}::new(#{AuthOptionListResolverParams}::new()));

                    #{additional_config}
                    Ok(())
                }
            }
            """,
            *codegenScope,
            "additional_config" to writable {
                writeCustomizations(
                    customizations,
                    OperationRuntimePluginSection.AdditionalConfig("cfg", operationShape),
                )
            },
        )
    }
}