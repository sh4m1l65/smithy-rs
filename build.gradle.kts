/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
buildscript {
    repositories {
        maven {
            url = uri("https://codeartifact-prod-862198903141.d.codeartifact.us-east-1.amazonaws.com/maven/codeartifact-prod-maven/")
            credentials {
                username = "aws"
                password = System.getenv("CODEARTIFACT_AUTH_TOKEN")
            }
        }
    }

    val kotlinVersion: String by project
    dependencies {
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:$kotlinVersion")
    }
}

allprojects {
    val allowLocalDeps: String by project
    repositories {
        maven {
            url = uri("https://codeartifact-prod-862198903141.d.codeartifact.us-east-1.amazonaws.com/maven/codeartifact-prod-maven/")
            credentials {
                username = "aws"
                password = System.getenv("CODEARTIFACT_AUTH_TOKEN")
            }
        }
    }
}

val ktlint by configurations.creating
val ktlintVersion: String by project

dependencies {
    ktlint("com.pinterest.ktlint:ktlint-cli:$ktlintVersion") {
        attributes {
            attribute(Bundling.BUNDLING_ATTRIBUTE, objects.named(Bundling.EXTERNAL))
        }
    }
}

val lintPaths =
    listOf(
        "**/*.kt",
        // Exclude build output directories
        "!**/build/**",
        "!**/node_modules/**",
        "!**/target/**",
    )

tasks.register<JavaExec>("ktlint") {
    description = "Check Kotlin code style."
    group = LifecycleBasePlugin.VERIFICATION_GROUP
    classpath = ktlint
    mainClass.set("com.pinterest.ktlint.Main")
    args = listOf("--log-level=info", "--relative", "--") + lintPaths
    // https://github.com/pinterest/ktlint/issues/1195#issuecomment-1009027802
    jvmArgs("--add-opens", "java.base/java.lang=ALL-UNNAMED")
}

tasks.register<JavaExec>("ktlintFormat") {
    description = "Auto fix Kotlin code style violations"
    group = LifecycleBasePlugin.VERIFICATION_GROUP
    classpath = ktlint
    mainClass.set("com.pinterest.ktlint.Main")
    args = listOf("--log-level=info", "--relative", "--format", "--") + lintPaths
    // https://github.com/pinterest/ktlint/issues/1195#issuecomment-1009027802
    jvmArgs("--add-opens", "java.base/java.lang=ALL-UNNAMED")
}

tasks.register<JavaExec>("ktlintPreCommit") {
    description = "Check Kotlin code style (for the pre-commit hooks)."
    group = LifecycleBasePlugin.VERIFICATION_GROUP
    classpath = ktlint
    mainClass.set("com.pinterest.ktlint.Main")
    args = listOf("--log-level=warn", "--color", "--relative", "--format", "--") +
        System.getProperty("ktlintPreCommitArgs").let { args ->
            check(args.isNotBlank()) { "need to pass in -DktlintPreCommitArgs=<some file paths to check>" }
            args.split(" ")
        }
    // https://github.com/pinterest/ktlint/issues/1195#issuecomment-1009027802
    jvmArgs("--add-opens", "java.base/java.lang=ALL-UNNAMED")
}
