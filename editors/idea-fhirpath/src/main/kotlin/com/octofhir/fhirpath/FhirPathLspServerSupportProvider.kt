package com.octofhir.fhirpath

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor

/**
 * LSP server support provider for FHIRPath.
 * Configures the connection to the fhirpath-lsp language server.
 */
class FhirPathLspServerSupportProvider : LspServerSupportProvider {

    override fun fileOpened(
        project: Project,
        file: VirtualFile,
        serverStarter: LspServerSupportProvider.LspServerStarter
    ) {
        if (file.extension == "fhirpath") {
            serverStarter.ensureServerStarted(FhirPathLspServerDescriptor(project))
        }
    }
}

/**
 * LSP server descriptor for FHIRPath.
 * Defines how to start and communicate with the fhirpath-lsp server.
 */
class FhirPathLspServerDescriptor(project: Project) : ProjectWideLspServerDescriptor(project, "FHIRPath") {

    override fun isSupportedFile(file: VirtualFile): Boolean {
        return file.extension == "fhirpath"
    }

    override fun createCommandLine(): com.intellij.execution.configurations.GeneralCommandLine {
        val commandLine = com.intellij.execution.configurations.GeneralCommandLine()

        // Find fhirpath-lsp in PATH
        commandLine.exePath = "fhirpath-lsp"

        // Set working directory to project root
        commandLine.workDirectory = project.basePath?.let { java.io.File(it) }

        return commandLine
    }

    override fun createInitializationOptions(): Any? {
        // Optional: Send initialization options to LSP server
        return mapOf(
            "fhirVersion" to "r5",
            "features" to mapOf(
                "diagnostics" to true,
                "completion" to true,
                "hover" to true,
                "inlayHints" to true
            )
        )
    }
}
