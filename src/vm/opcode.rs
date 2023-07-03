use crate::vm::Builder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
	LoadConstant(usize),
	LoadArgument(usize),
	LoadVariable(usize),

	GenericCall(usize),

	CreatePath(usize),
	CreateRegex(usize),
	CreateString(usize),

	Not,
	Negate,
	UPositive,
	ForcedLogical,

	Add,
	Subtract,
	Multiply,
	Divide,

	// Querying
	IsFile { implicit: bool },
	IsDirectory { implicit: bool },
	IsExecutable { implicit: bool },
	IsSymlink { implicit: bool },
	IsBinary { implicit: bool },
	IsHidden { implicit: bool },
	IsGitIgnored { implicit: bool },
	IsOk(usize),

	// Path-related funcitons
	PushRoot,
	PushPath,
	PushPwd,
	Dirname { implicit: bool },
	Extname { implicit: bool },
	ExtnameDot { implicit: bool },
	Basename { implicit: bool },
	Stemname { implicit: bool },

	// Misc
	Print(usize),
	Write(usize), // same as print just no newline at end
	Skip,
	Quit { implicit: bool },
	Depth { implicit: bool },
	Sleep { implicit: bool },

	// Interactive
	Mv { implicit: bool, force: Option<bool> },
	Rm { implicit: bool, force: Option<bool> },
	RmR { implicit: bool, force: Option<bool> },
	Cp { implicit: bool, force: Option<bool> },
	Ln { implicit: bool, force: Option<bool> },
	LnS { implicit: bool, force: Option<bool> },
	Mkdir,
	Touch { implicit: bool },
}

impl Opcode {
	pub fn compile_fn_call(name: &str, argc: usize, builder: &mut Builder) -> bool {
		macro_rules! implicit {
			($name:ident) => {
				implicit!($name, 0)
			};
			($name:ident, $amount:literal $($rest:tt)*) => {
				builder.opcode(Self::$name { implicit: argc == $amount $($rest)* })
			};
		}
		match (name, argc) {
			// Querying
			("f?" | "file?" | "isfile", 0 | 1) => implicit!(IsFile),
			("d?" | "dir?" | "directory?" | "isdir", 0 | 1) => implicit!(IsDirectory),
			("e?" | "exe?" | "executable?" | "isexe", 0 | 1) => implicit!(IsExecutable),
			("s?" | "sym?" | "symlink?" | "issym", 0 | 1) => implicit!(IsSymlink),
			("b?" | "bin?" | "binary?" | "isbin", 0 | 1) => implicit!(IsBinary),
			("gi?" | "gitignore?" | "gitignored?" | "isgi", 0 | 1) => implicit!(IsGitIgnored),
			("h?" | "hidden?" | "dot?" | "ishidden" | "isdot", 0 | 1) => implicit!(IsHidden),
			("ok?", 1..) => builder.opcode(Opcode::IsOk(argc)),

			// Path-related funcitons
			("r" | "root", 0) => builder.opcode(Self::PushRoot),
			("p" | "path", 0) => builder.opcode(Self::PushPath),
			("pwd", 0) => builder.opcode(Self::PushPwd),
			("d" | "dir" | "dirname" | "directory" | "parent", 0 | 1) => implicit!(Dirname),
			("e" | "ext" | "extname" | "extension", 0 | 1) => implicit!(Extname),
			("ed" | "extd" | "extnamed" | "extnamedot" | "extensiond", 0 | 1) => implicit!(ExtnameDot),
			("b" | "base" | "basename", 0 | 1) => implicit!(Basename),
			("s" | "stem" | "stemname", 0 | 1) => implicit!(Stemname),

			// Misc
			("pr" | "print", _) => builder.opcode(Self::Print(argc)),
			("wr" | "write", _) => builder.opcode(Self::Write(argc)),
			("next" | "skip", 0) => builder.opcode(Self::Skip),
			("q" | "quit" | "exit", 0 | 1) => implicit!(Quit),
			("depth", 1 | 2) => implicit!(Depth, 1),
			("date", _) => todo!(),
			("sleep", 0 | 1) => implicit!(Sleep),

			// Executable functions
			("exec", _) => todo!(),
			("mv" | "mvf" | "mvi", 1 | 2) => {
				implicit!(Mv, 1, force: (name != "mv").then_some(name == "mvf"))
			}
			("rm" | "rmf" | "rmi", 1 | 2) => {
				implicit!(Rm, 1, force: (name != "rm").then_some(name == "rmf"))
			}
			("rmr" | "rmrf" | "rmri", 1 | 2) => {
				implicit!(RmR, 1, force: (name != "rmr").then_some(name == "rmrf"))
			}
			("cp" | "cpf" | "cpi", 1 | 2) => {
				implicit!(Cp, 1, force: (name != "cp").then_some(name == "cpf"))
			}
			("ln" | "lnf" | "lni", 1 | 2) => {
				implicit!(Ln, 1, force: (name != "ln").then_some(name == "lnf"))
			}
			("lns" | "lnsf" | "lnsi", 1 | 2) => {
				implicit!(LnS, 1, force: (name != "lns").then_some(name == "lnsf"))
			}
			("mkdir", 1) => builder.opcode(Self::Mkdir),
			("t" | "touch", 0 | 1) => implicit!(Touch),
			_ => return false,
		}

		true
	}
}
