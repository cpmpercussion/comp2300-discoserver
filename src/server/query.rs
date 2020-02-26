#[derive(Debug)]
pub enum Query {
    CurrentThread,
    CrcChecksum { addr: u32, length: u32 },
    ThreadInfoFirst,
    ThreadInfoSubsequent,
    ThreadInfoDeprecated { first: bool, count: u32, next_thread: u32 }, // qL
    ThreadLocalStorageAddress { id: i32, offset: i32, lm: u32 },
    ThreadInformationBlockAddress { id: i32 },
    SectionOffsets,
    ThreadExtraInfoDeprecated { mode: u32, id: i32 },
    ExecCommand { command: Vec<u8> },
    SearchMemory { address: u32, length: u32, pattern: Vec<u8> },
    Supported { features: Vec<GdbFeature> },
    OfferingSymbolLookup,
    TracepointStatus, // qTStatus
    TracepointPieceFirst, // qTfP
    TracepointPieceSubsequent, // qTsP
    TracevariableFirst, // qTfV
    TracevariableSubsequent, // qTsV
    TracepointStaticFirst, // qTfSTM
    TracepointStaticSubsequent, // qTsSTM
    TracepointStaticAtAddress, // qTSTMat:address
    TracepointBuffer { offset: u32, length: u32 },
    ThreadExtraInfo { id: i32 },
    AttachedToProcess { process: Option<u32> },
}

#[derive(Debug)]
pub enum AllowOp {
    WriteReg,
    WriteMem,
    InsertBreak,
    InsertTrace,
    InsertFastTrace,
    Stop,
}

#[derive(Debug)]
pub enum GdbFeature {
    SoftwareBreakpoint { supported: bool },
    HardwareBreakpoint { supported: bool },
    VContSupported { supported: bool },
}

#[derive(Debug)]
pub enum ServerFeature {
    PacketSize { size: u32 },
    NoAcknowledgmentMode { supported: bool },
    SoftwareBreakpoint { supported: bool },
    HardwareBreakpoint { supported: bool },
    VContSupported { supported: bool },
}

#[derive(Debug)]
pub enum BranchTraceKind {
    BranchTraceStore,
    IntelProcessorTrace,
    Disable,
}

#[derive(Debug)]
pub enum Set {
    Agent { enabled: bool },
    Allow { operations: Vec<(AllowOp, bool)> },
    DisableRandomization { disabled: bool },
    StartupWithShell { enabled: bool },
    EnvironmentHexEncoded { name: String, value: String },
    EnvironmentUnset { name: String, value: String },
    EnvironmentReset,
    WorkingDir { dir: String },
    NonStop { enabled: bool },
    CatchSyscalls { enabled: bool, calls: Vec<u32> },
    PassSignals { signals: Vec<u32> },
    ProgramSignals { signals: Vec<u32> },
    ThreadEvents { enabled: bool },
    NoAcknowledgmentMode,
    TracepointSave { file: String },
    TracepointBufferKind { circular: bool },
    TracepointBufferConstruct { size: i32 },
    TracepointNotes,
    BranchTracing { kind: BranchTraceKind },
    BranchTracingBufferSize { kind: BranchTraceKind, size: u32 }
}
