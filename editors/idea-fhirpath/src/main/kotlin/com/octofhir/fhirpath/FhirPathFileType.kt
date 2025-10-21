package com.octofhir.fhirpath

import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

object FhirPathFileType : LanguageFileType(FhirPathLanguage) {
    override fun getName(): String = "FHIRPath"

    override fun getDescription(): String = "FHIRPath expression file"

    override fun getDefaultExtension(): String = "fhirpath"

    override fun getIcon(): Icon? = null // TODO: Add custom icon

    override fun getDisplayName(): String = "FHIRPath"
}
