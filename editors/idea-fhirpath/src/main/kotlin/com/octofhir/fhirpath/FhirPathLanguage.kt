package com.octofhir.fhirpath

import com.intellij.lang.Language

object FhirPathLanguage : Language("FHIRPath") {
    override fun getDisplayName(): String = "FHIRPath"

    override fun isCaseSensitive(): Boolean = true
}
