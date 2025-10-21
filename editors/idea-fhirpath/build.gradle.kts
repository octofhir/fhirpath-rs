plugins {
    id("org.jetbrains.intellij.platform") version "2.9.0"
    kotlin("jvm") version "1.9.0"
}

group = "com.octofhir"
version = "0.1.0"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        intellijIdeaUltimate("2023.2")
        instrumentationTools()
    }
}

intellijPlatform {
    pluginConfiguration {
        name = "FHIRPath"
        version = project.version.toString()
        description = """
            FHIRPath language support with LSP integration.

            Features:
            - Syntax highlighting
            - Real-time diagnostics
            - Code completion
            - Hover documentation
            - Inlay hints
            - Code actions
            - Go to definition
        """.trimIndent()

        ideaVersion {
            sinceBuild = "232"
            untilBuild = "242.*"
        }
    }

    publishing {
        token = providers.environmentVariable("PUBLISH_TOKEN")
    }
}

tasks {
    compileKotlin {
        kotlinOptions.jvmTarget = "17"
    }

    buildPlugin {
        archiveBaseName.set("fhirpath-idea-plugin")
    }
}
