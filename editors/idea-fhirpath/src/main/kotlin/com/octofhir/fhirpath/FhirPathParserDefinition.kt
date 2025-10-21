package com.octofhir.fhirpath

import com.intellij.lang.ASTNode
import com.intellij.lang.ParserDefinition
import com.intellij.lang.PsiParser
import com.intellij.lexer.Lexer
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IFileElementType
import com.intellij.psi.tree.TokenSet

/**
 * Basic parser definition for FHIRPath.
 * Note: Actual parsing is handled by the LSP server.
 * This is a minimal implementation to satisfy IntelliJ's requirements.
 */
class FhirPathParserDefinition : ParserDefinition {

    companion object {
        val FILE = IFileElementType(FhirPathLanguage)
    }

    override fun createLexer(project: Project?): Lexer {
        return FhirPathLexer()
    }

    override fun createParser(project: Project?): PsiParser {
        return FhirPathParser()
    }

    override fun getFileNodeType(): IFileElementType = FILE

    override fun getCommentTokens(): TokenSet = TokenSet.EMPTY

    override fun getStringLiteralElements(): TokenSet = TokenSet.EMPTY

    override fun createElement(node: ASTNode?): PsiElement {
        return FhirPathPsiElement(node!!)
    }

    override fun createFile(viewProvider: FileViewProvider): PsiFile {
        return FhirPathFile(viewProvider)
    }
}

/**
 * Minimal lexer - LSP handles actual tokenization
 */
class FhirPathLexer : Lexer() {
    private var buffer: CharSequence? = null
    private var startOffset: Int = 0
    private var endOffset: Int = 0
    private var state: Int = 0

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        this.buffer = buffer
        this.startOffset = startOffset
        this.endOffset = endOffset
        this.state = initialState
    }

    override fun getState(): Int = state

    override fun getTokenType() = null

    override fun getTokenStart(): Int = startOffset

    override fun getTokenEnd(): Int = endOffset

    override fun advance() {
        startOffset = endOffset
    }

    override fun getBufferSequence(): CharSequence = buffer ?: ""

    override fun getBufferEnd(): Int = endOffset
}

/**
 * Minimal parser - LSP handles actual parsing
 */
class FhirPathParser : PsiParser {
    override fun parse(root: IFileElementType, builder: com.intellij.lang.PsiBuilder): ASTNode {
        val marker = builder.mark()
        while (!builder.eof()) {
            builder.advanceLexer()
        }
        marker.done(root)
        return builder.treeBuilt
    }
}

/**
 * PSI element for FHIRPath
 */
class FhirPathPsiElement(node: ASTNode) : com.intellij.extapi.psi.ASTWrapperPsiElement(node)

/**
 * PSI file for FHIRPath
 */
class FhirPathFile(viewProvider: FileViewProvider) :
    com.intellij.extapi.psi.PsiFileBase(viewProvider, FhirPathLanguage) {

    override fun getFileType() = FhirPathFileType

    override fun toString(): String = "FHIRPath File"
}
