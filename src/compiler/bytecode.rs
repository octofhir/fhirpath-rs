//! Bytecode instruction set for FHIRPath expressions
//!
//! This module defines a compact bytecode representation for FHIRPath expressions
//! that enables faster execution than AST interpretation.

use crate::model::FhirPathValue;
use std::fmt;

/// Index into the constant pool
pub type ConstantIndex = u16;

/// Index into the string pool  
pub type StringIndex = u16;

/// Index into the function registry
pub type FunctionIndex = u16;

/// Bytecode instruction set for FHIRPath expressions
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    // === Stack Operations ===
    /// Push a constant value onto the stack
    /// Operand: index into constant pool
    PushConstant(ConstantIndex),

    /// Push the current input value onto the stack
    PushInput,

    /// Duplicate the top stack value
    Duplicate,

    /// Pop and discard the top stack value
    Pop,

    /// Swap the top two stack values
    Swap,

    // === Property Access ===
    /// Load a property by name from the top stack value
    /// Operand: index into string pool
    LoadProperty(StringIndex),

    /// Load a property with index access (e.g., name[0])
    /// Stack: [index, object] -> [result]
    LoadIndexedProperty(StringIndex),

    /// Access collection element by index
    /// Stack: [index, collection] -> [element]
    IndexAccess,

    // === Function Calls ===
    /// Call a function with specified arity
    /// Operands: function index, argument count
    /// Stack: [arg1, arg2, ..., argN] -> [result]
    CallFunction(FunctionIndex, u8),

    /// Call a method on the top stack object
    /// Operands: method name, argument count
    /// Stack: [obj, arg1, ..., argN] -> [result]
    CallMethod(StringIndex, u8),

    // === Arithmetic Operations ===
    /// Add two values from stack
    /// Stack: [left, right] -> [result]
    Add,

    /// Subtract two values from stack
    /// Stack: [left, right] -> [result]
    Subtract,

    /// Multiply two values from stack
    /// Stack: [left, right] -> [result]
    Multiply,

    /// Divide two values from stack
    /// Stack: [left, right] -> [result]
    Divide,

    /// Modulo operation
    /// Stack: [left, right] -> [result]
    Modulo,

    /// Negate the top stack value
    /// Stack: [value] -> [-value]
    Negate,

    // === Comparison Operations ===
    /// Equal comparison
    /// Stack: [left, right] -> [boolean]
    Equal,

    /// Not equal comparison
    /// Stack: [left, right] -> [boolean]
    NotEqual,

    /// Less than comparison
    /// Stack: [left, right] -> [boolean]
    LessThan,

    /// Less than or equal comparison
    /// Stack: [left, right] -> [boolean]
    LessThanOrEqual,

    /// Greater than comparison
    /// Stack: [left, right] -> [boolean]
    GreaterThan,

    /// Greater than or equal comparison
    /// Stack: [left, right] -> [boolean]
    GreaterThanOrEqual,

    // === Logical Operations ===
    /// Logical AND
    /// Stack: [left, right] -> [result]
    And,

    /// Logical OR
    /// Stack: [left, right] -> [result]
    Or,

    /// Logical NOT
    /// Stack: [value] -> [!value]
    Not,

    // === Collection Operations ===
    /// Create a collection from top N stack values
    /// Operand: number of elements
    /// Stack: [elem1, elem2, ..., elemN] -> [collection]
    MakeCollection(u8),

    /// Union two collections
    /// Stack: [left, right] -> [union]
    Union,

    /// Flatten a nested collection
    /// Stack: [collection] -> [flattened]
    Flatten,

    /// Check if collection is empty
    /// Stack: [collection] -> [boolean]
    IsEmpty,

    /// Get collection count
    /// Stack: [collection] -> [count]
    Count,

    /// Check if any element satisfies condition
    /// Stack: [collection] -> [boolean]
    Any,

    /// Check if all elements satisfy condition
    /// Stack: [collection] -> [boolean]
    All,

    // === Control Flow ===
    /// Jump unconditionally
    /// Operand: relative offset (signed)
    Jump(i16),

    /// Jump if top stack value is false
    /// Operand: relative offset (signed)
    /// Stack: [condition] -> []
    JumpIfFalse(i16),

    /// Jump if top stack value is true
    /// Operand: relative offset (signed)
    /// Stack: [condition] -> []
    JumpIfTrue(i16),

    // === Lambda Operations ===
    /// Begin lambda scope
    /// Creates new variable scope for lambda parameters
    BeginLambda,

    /// End lambda scope
    /// Restores previous variable scope
    EndLambda,

    /// Bind lambda parameter
    /// Operand: parameter name index
    /// Stack: [value] -> []
    BindParameter(StringIndex),

    // === Variable Operations ===
    /// Load variable value
    /// Operand: variable name index
    /// Stack: [] -> [value]
    LoadVariable(StringIndex),

    /// Store variable value
    /// Operand: variable name index
    /// Stack: [value] -> []
    StoreVariable(StringIndex),

    // === Type Operations ===
    /// Type checking (is operator)
    /// Operand: type name index
    /// Stack: [value] -> [boolean]
    IsType(StringIndex),

    /// Type casting (as operator)
    /// Operand: type name index
    /// Stack: [value] -> [cast_result]
    AsType(StringIndex),

    // === Specialized Operations ===
    /// Filter collection with predicate
    /// The predicate bytecode follows this instruction
    /// Stack: [collection] -> [filtered]
    Filter,

    /// Select/transform collection elements
    /// The transform bytecode follows this instruction
    /// Stack: [collection] -> [transformed]
    Select,

    /// Where clause (alias for Filter for clarity)
    Where,

    // === Optimization Instructions ===
    /// No-operation (for alignment and optimization)
    Nop,

    /// Return from function/expression
    /// Stack: [result] -> [result] (VM stops)
    Return,

    /// Fast path for simple property access
    /// Operand: property name index
    /// Stack: [object] -> [property_value]
    FastProperty(StringIndex),

    /// Fast path for literal values (pre-evaluated)
    /// Operand: constant index
    /// Stack: [] -> [constant]
    FastConstant(ConstantIndex),
}

impl Instruction {
    /// Get the size in bytes of this instruction including operands
    pub fn size(&self) -> usize {
        match self {
            // Instructions with no operands
            Self::PushInput
            | Self::Duplicate
            | Self::Pop
            | Self::Swap
            | Self::IndexAccess
            | Self::Add
            | Self::Subtract
            | Self::Multiply
            | Self::Divide
            | Self::Modulo
            | Self::Negate
            | Self::Equal
            | Self::NotEqual
            | Self::LessThan
            | Self::LessThanOrEqual
            | Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::And
            | Self::Or
            | Self::Not
            | Self::Union
            | Self::Flatten
            | Self::IsEmpty
            | Self::Count
            | Self::Any
            | Self::All
            | Self::BeginLambda
            | Self::EndLambda
            | Self::Filter
            | Self::Select
            | Self::Where
            | Self::Nop
            | Self::Return => 1,

            // Instructions with 1-byte operands
            Self::MakeCollection(_) => 2,

            // Instructions with 2-byte operands
            Self::PushConstant(_)
            | Self::LoadProperty(_)
            | Self::LoadIndexedProperty(_)
            | Self::BindParameter(_)
            | Self::LoadVariable(_)
            | Self::StoreVariable(_)
            | Self::IsType(_)
            | Self::AsType(_)
            | Self::FastProperty(_)
            | Self::FastConstant(_)
            | Self::Jump(_)
            | Self::JumpIfFalse(_)
            | Self::JumpIfTrue(_) => 3,

            // Instructions with function index (2 bytes) + arity (1 byte)
            Self::CallFunction(_, _) | Self::CallMethod(_, _) => 4,
        }
    }

    /// Check if this instruction modifies the stack
    pub fn modifies_stack(&self) -> bool {
        !matches!(self, Self::Nop)
    }

    /// Get the stack effect of this instruction (positive = pushes, negative = pops)
    pub fn stack_effect(&self) -> i8 {
        match self {
            // Push operations
            Self::PushConstant(_) | Self::PushInput | Self::Duplicate | Self::FastConstant(_) => 1,

            // Load operations
            Self::LoadProperty(_)
            | Self::LoadIndexedProperty(_)
            | Self::LoadVariable(_)
            | Self::FastProperty(_) => 0, // pop + push = 0

            // Pop operations
            Self::Pop | Self::StoreVariable(_) | Self::BindParameter(_) => -1,

            // Swap has no net effect
            Self::Swap => 0,

            // Binary operations (pop 2, push 1)
            Self::Add
            | Self::Subtract
            | Self::Multiply
            | Self::Divide
            | Self::Modulo
            | Self::Equal
            | Self::NotEqual
            | Self::LessThan
            | Self::LessThanOrEqual
            | Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::And
            | Self::Or
            | Self::Union
            | Self::IndexAccess => -1,

            // Unary operations (pop 1, push 1)
            Self::Negate
            | Self::Not
            | Self::Flatten
            | Self::IsEmpty
            | Self::Count
            | Self::Any
            | Self::All
            | Self::IsType(_)
            | Self::AsType(_)
            | Self::Filter
            | Self::Select
            | Self::Where => 0,

            // Function calls
            Self::CallFunction(_, arity) => 1 - (*arity as i8),
            Self::CallMethod(_, arity) => -((*arity as i8) + 1) + 1, // pop object + args, push result

            // Collection creation
            Self::MakeCollection(count) => 1 - (*count as i8),

            // Control flow
            Self::Jump(_) => 0,
            Self::JumpIfFalse(_) | Self::JumpIfTrue(_) => -1,

            // Lambda operations
            Self::BeginLambda | Self::EndLambda => 0,

            // Optimization instructions
            Self::Nop | Self::Return => 0,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PushConstant(idx) => write!(f, "PUSH_CONST {}", idx),
            Self::PushInput => write!(f, "PUSH_INPUT"),
            Self::Duplicate => write!(f, "DUP"),
            Self::Pop => write!(f, "POP"),
            Self::Swap => write!(f, "SWAP"),
            Self::LoadProperty(idx) => write!(f, "LOAD_PROP {}", idx),
            Self::LoadIndexedProperty(idx) => write!(f, "LOAD_INDEXED_PROP {}", idx),
            Self::IndexAccess => write!(f, "INDEX"),
            Self::CallFunction(func_idx, arity) => write!(f, "CALL {} {}", func_idx, arity),
            Self::CallMethod(name_idx, arity) => write!(f, "CALL_METHOD {} {}", name_idx, arity),
            Self::Add => write!(f, "ADD"),
            Self::Subtract => write!(f, "SUB"),
            Self::Multiply => write!(f, "MUL"),
            Self::Divide => write!(f, "DIV"),
            Self::Modulo => write!(f, "MOD"),
            Self::Negate => write!(f, "NEG"),
            Self::Equal => write!(f, "EQ"),
            Self::NotEqual => write!(f, "NE"),
            Self::LessThan => write!(f, "LT"),
            Self::LessThanOrEqual => write!(f, "LE"),
            Self::GreaterThan => write!(f, "GT"),
            Self::GreaterThanOrEqual => write!(f, "GE"),
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
            Self::Not => write!(f, "NOT"),
            Self::MakeCollection(count) => write!(f, "MAKE_COLLECTION {}", count),
            Self::Union => write!(f, "UNION"),
            Self::Flatten => write!(f, "FLATTEN"),
            Self::IsEmpty => write!(f, "IS_EMPTY"),
            Self::Count => write!(f, "COUNT"),
            Self::Any => write!(f, "ANY"),
            Self::All => write!(f, "ALL"),
            Self::Jump(offset) => write!(f, "JUMP {}", offset),
            Self::JumpIfFalse(offset) => write!(f, "JMP_FALSE {}", offset),
            Self::JumpIfTrue(offset) => write!(f, "JMP_TRUE {}", offset),
            Self::BeginLambda => write!(f, "BEGIN_LAMBDA"),
            Self::EndLambda => write!(f, "END_LAMBDA"),
            Self::BindParameter(idx) => write!(f, "BIND_PARAM {}", idx),
            Self::LoadVariable(idx) => write!(f, "LOAD_VAR {}", idx),
            Self::StoreVariable(idx) => write!(f, "STORE_VAR {}", idx),
            Self::IsType(idx) => write!(f, "IS_TYPE {}", idx),
            Self::AsType(idx) => write!(f, "AS_TYPE {}", idx),
            Self::Filter => write!(f, "FILTER"),
            Self::Select => write!(f, "SELECT"),
            Self::Where => write!(f, "WHERE"),
            Self::Nop => write!(f, "NOP"),
            Self::Return => write!(f, "RETURN"),
            Self::FastProperty(idx) => write!(f, "FAST_PROP {}", idx),
            Self::FastConstant(idx) => write!(f, "FAST_CONST {}", idx),
        }
    }
}

/// Bytecode program containing instructions and constant pools
#[derive(Debug, Clone)]
pub struct Bytecode {
    /// Instruction sequence
    pub instructions: Vec<Instruction>,

    /// Constant value pool
    pub constants: Vec<FhirPathValue>,

    /// String constant pool (for property names, function names, etc.)
    pub strings: Vec<String>,

    /// Maximum stack depth required for execution
    pub max_stack_depth: usize,

    /// Metadata for debugging and optimization
    pub metadata: BytecodeMetadata,
}

/// Metadata associated with bytecode
#[derive(Debug, Clone, Default)]
pub struct BytecodeMetadata {
    /// Original expression string (for debugging)
    pub source: Option<String>,

    /// Optimization level used during compilation
    pub optimization_level: OptimizationLevel,

    /// Whether this bytecode uses lambdas
    pub uses_lambdas: bool,

    /// Whether this bytecode modifies variables
    pub modifies_variables: bool,

    /// Estimated complexity score
    pub complexity_score: u32,
}

/// Bytecode optimization levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptimizationLevel {
    /// No optimizations
    None,

    /// Basic optimizations (constant folding, dead code elimination)
    #[default]
    Basic,

    /// Advanced optimizations (loop unrolling, function inlining)
    Advanced,

    /// Aggressive optimizations (speculative optimizations)
    Aggressive,
}

impl Bytecode {
    /// Create new empty bytecode
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            strings: Vec::new(),
            max_stack_depth: 0,
            metadata: BytecodeMetadata::default(),
        }
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, value: FhirPathValue) -> ConstantIndex {
        // Check if constant already exists to avoid duplicates
        if let Some(index) = self.constants.iter().position(|v| v == &value) {
            return index as ConstantIndex;
        }

        let index = self.constants.len();
        self.constants.push(value);
        index as ConstantIndex
    }

    /// Add a string to the pool and return its index
    pub fn add_string(&mut self, string: String) -> StringIndex {
        // Check if string already exists to avoid duplicates
        if let Some(index) = self.strings.iter().position(|s| s == &string) {
            return index as StringIndex;
        }

        let index = self.strings.len();
        self.strings.push(string);
        index as StringIndex
    }

    /// Add an instruction to the bytecode
    pub fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    /// Calculate the maximum stack depth required
    pub fn calculate_max_stack_depth(&mut self) {
        let mut current_depth = 0i32;
        let mut max_depth = 0i32;

        for instruction in &self.instructions {
            current_depth += instruction.stack_effect() as i32;
            max_depth = max_depth.max(current_depth);
        }

        self.max_stack_depth = max_depth.max(0) as usize;
    }

    /// Get the size in bytes of the bytecode
    pub fn size(&self) -> usize {
        self.instructions.iter().map(|i| i.size()).sum()
    }

    /// Pretty print the bytecode for debugging
    pub fn disassemble(&self) -> String {
        let mut output = String::new();
        output.push_str("=== BYTECODE DISASSEMBLY ===\n");

        if let Some(source) = &self.metadata.source {
            output.push_str(&format!("Source: {}\n", source));
        }

        output.push_str(&format!(
            "Optimization Level: {:?}\n",
            self.metadata.optimization_level
        ));
        output.push_str(&format!("Max Stack Depth: {}\n", self.max_stack_depth));
        output.push_str(&format!("Constants: {}\n", self.constants.len()));
        output.push_str(&format!("Strings: {}\n", self.strings.len()));
        output.push_str("\n--- CONSTANTS ---\n");

        for (i, constant) in self.constants.iter().enumerate() {
            output.push_str(&format!("{:4}: {:?}\n", i, constant));
        }

        output.push_str("\n--- STRINGS ---\n");
        for (i, string) in self.strings.iter().enumerate() {
            output.push_str(&format!("{:4}: \"{}\"\n", i, string));
        }

        output.push_str("\n--- INSTRUCTIONS ---\n");
        for (i, instruction) in self.instructions.iter().enumerate() {
            output.push_str(&format!("{:4}: {}\n", i, instruction));
        }

        output
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

/// Bytecode builder utility for constructing bytecode programs
pub struct BytecodeBuilder {
    bytecode: Bytecode,
    label_targets: std::collections::HashMap<String, usize>,
    pending_jumps: Vec<(usize, String, JumpType)>,
}

/// Type of jump instruction for label resolution
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum JumpType {
    Unconditional,
    IfFalse,
    IfTrue,
}

impl BytecodeBuilder {
    /// Create a new bytecode builder
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            label_targets: std::collections::HashMap::new(),
            pending_jumps: Vec::new(),
        }
    }

    /// Emit an instruction
    pub fn emit(&mut self, instruction: Instruction) -> &mut Self {
        self.bytecode.emit(instruction);
        self
    }

    /// Add a constant and emit PushConstant instruction
    pub fn push_constant(&mut self, value: FhirPathValue) -> &mut Self {
        let index = self.bytecode.add_constant(value);
        self.emit(Instruction::PushConstant(index))
    }

    /// Add a string and emit LoadProperty instruction
    pub fn load_property(&mut self, property_name: String) -> &mut Self {
        let index = self.bytecode.add_string(property_name);
        self.emit(Instruction::LoadProperty(index))
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, value: FhirPathValue) -> u16 {
        self.bytecode.add_constant(value)
    }

    /// Add a string to the pool and return its index
    pub fn add_string(&mut self, string: String) -> u16 {
        self.bytecode.add_string(string)
    }

    /// Create a label at the current position
    pub fn label(&mut self, name: String) -> &mut Self {
        let position = self.bytecode.instructions.len();
        self.label_targets.insert(name, position);
        self
    }

    /// Emit a jump to a label (to be resolved later)
    pub fn jump_to(&mut self, label: String) -> &mut Self {
        let position = self.bytecode.instructions.len();
        self.pending_jumps
            .push((position, label, JumpType::Unconditional));
        self.emit(Instruction::Jump(0)) // Placeholder offset
    }

    /// Emit a conditional jump to a label
    pub fn jump_if_false_to(&mut self, label: String) -> &mut Self {
        let position = self.bytecode.instructions.len();
        self.pending_jumps
            .push((position, label, JumpType::IfFalse));
        self.emit(Instruction::JumpIfFalse(0)) // Placeholder offset
    }

    /// Finalize the bytecode by resolving jumps and calculating stack depth
    pub fn finalize(mut self) -> Result<Bytecode, String> {
        // Resolve pending jumps
        for (instruction_pos, label, jump_type) in self.pending_jumps {
            let target_pos = self
                .label_targets
                .get(&label)
                .ok_or_else(|| format!("Undefined label: {}", label))?;

            let offset = *target_pos as i32 - instruction_pos as i32;
            if offset < i16::MIN as i32 || offset > i16::MAX as i32 {
                return Err(format!("Jump offset too large: {}", offset));
            }

            let new_instruction = match jump_type {
                JumpType::Unconditional => Instruction::Jump(offset as i16),
                JumpType::IfFalse => Instruction::JumpIfFalse(offset as i16),
                JumpType::IfTrue => Instruction::JumpIfTrue(offset as i16),
            };

            self.bytecode.instructions[instruction_pos] = new_instruction;
        }

        // Calculate maximum stack depth
        self.bytecode.calculate_max_stack_depth();

        Ok(self.bytecode)
    }
}

impl Default for BytecodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FhirPathValue;

    #[test]
    fn test_instruction_size() {
        assert_eq!(Instruction::PushInput.size(), 1);
        assert_eq!(Instruction::PushConstant(0).size(), 3);
        assert_eq!(Instruction::CallFunction(0, 2).size(), 4);
        assert_eq!(Instruction::Jump(0).size(), 3);
    }

    #[test]
    fn test_stack_effect() {
        assert_eq!(Instruction::PushConstant(0).stack_effect(), 1);
        assert_eq!(Instruction::Pop.stack_effect(), -1);
        assert_eq!(Instruction::Add.stack_effect(), -1); // pop 2, push 1
        assert_eq!(Instruction::CallFunction(0, 2).stack_effect(), -1); // pop 2 args, push 1 result
    }

    #[test]
    fn test_bytecode_builder() {
        let mut builder = BytecodeBuilder::new();
        builder
            .push_constant(FhirPathValue::Integer(42))
            .load_property("name".to_string())
            .emit(Instruction::Add);

        let bytecode = builder.finalize().unwrap();

        assert_eq!(bytecode.instructions.len(), 3);
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.strings.len(), 1);
        assert!(bytecode.max_stack_depth > 0);
    }

    #[test]
    fn test_constant_deduplication() {
        let mut bytecode = Bytecode::new();

        let idx1 = bytecode.add_constant(FhirPathValue::Integer(42));
        let idx2 = bytecode.add_constant(FhirPathValue::Integer(42));

        assert_eq!(idx1, idx2);
        assert_eq!(bytecode.constants.len(), 1);
    }

    #[test]
    fn test_string_deduplication() {
        let mut bytecode = Bytecode::new();

        let idx1 = bytecode.add_string("name".to_string());
        let idx2 = bytecode.add_string("name".to_string());

        assert_eq!(idx1, idx2);
        assert_eq!(bytecode.strings.len(), 1);
    }

    #[test]
    fn test_max_stack_depth_calculation() {
        let mut bytecode = Bytecode::new();
        bytecode.emit(Instruction::PushConstant(0)); // depth: 1
        bytecode.emit(Instruction::PushConstant(1)); // depth: 2
        bytecode.emit(Instruction::Add); // depth: 1
        bytecode.calculate_max_stack_depth();

        assert_eq!(bytecode.max_stack_depth, 2);
    }

    #[test]
    fn test_instruction_display() {
        assert_eq!(
            format!("{}", Instruction::PushConstant(42)),
            "PUSH_CONST 42"
        );
        assert_eq!(format!("{}", Instruction::CallFunction(1, 3)), "CALL 1 3");
        assert_eq!(format!("{}", Instruction::Add), "ADD");
    }

    #[test]
    fn test_bytecode_disassembly() {
        let mut bytecode = Bytecode::new();
        bytecode.add_constant(FhirPathValue::Integer(42));
        bytecode.add_string("name".to_string());
        bytecode.emit(Instruction::PushConstant(0));
        bytecode.emit(Instruction::LoadProperty(0));
        bytecode.metadata.source = Some("Patient.name".to_string());

        let disassembly = bytecode.disassemble();
        assert!(disassembly.contains("Source: Patient.name"));
        assert!(disassembly.contains("PUSH_CONST 0"));
        assert!(disassembly.contains("LOAD_PROP 0"));
    }
}
