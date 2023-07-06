use crate::vm::Builder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
	Illegal,
	LoadConstant(usize),
	LoadArgument(usize),
	LoadVariable(usize),
	StoreArgument(usize),
	StoreVariable(usize),

	Dup,
	Pop,

	GenericCall(usize),

	CreatePath(usize),
	CreateRegex(usize),
	CreateString(usize),

	Return,
	Jump(usize),
	JumpIf(usize),
	JumpUnless(usize),

	Not,
	Negate,
	UPositive,
	ForcedLogical,

	Add,
	Subtract,
	Multiply,
	Divide,
	Modulo,

	Matches,
	NotMatches,
	Equal,
	NotEqual,
	LessThan,
	LessThanOrEqual,
	GreaterThan,
	GreaterThanOrEqual,

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
	FileSize { implicit: bool },
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
	pub fn arity(self) -> usize {
		use Opcode::*;
		match self {
			Illegal => unreachable!(),
			LoadConstant(_) | LoadArgument(_) | LoadVariable(_) => 0,
			StoreArgument(_) | StoreVariable(_) => 1,

			Dup | Pop => 1,

			GenericCall(argc) => argc + 1,
			CreatePath(_num) => todo!(),
			CreateRegex(_num) => todo!(),
			CreateString(_num) => todo!(),

			Return => 1,
			Jump(_) => 0,
			JumpIf(_) | JumpUnless(_) => 1,

			Not | Negate | UPositive | ForcedLogical => 1,

			Add | Subtract | Multiply | Divide | Modulo => 2,
			Matches | NotMatches | Equal | NotEqual | LessThan | LessThanOrEqual | GreaterThan
			| GreaterThanOrEqual => 2,

			// Querying
			IsFile { implicit }
			| IsDirectory { implicit }
			| IsExecutable { implicit }
			| IsSymlink { implicit }
			| IsBinary { implicit }
			| IsHidden { implicit }
			| IsGitIgnored { implicit } => !implicit as usize,
			IsOk(argc) => argc,

			// Path-related funcitons
			PushRoot | PushPath | PushPwd => 0,
			FileSize { implicit }
			| Dirname { implicit }
			| Extname { implicit }
			| ExtnameDot { implicit }
			| Basename { implicit }
			| Stemname { implicit } => !implicit as usize,

			// Misc
			Print(argc) | Write(argc) => argc,
			Skip => todo!(),
			Quit { implicit } => !implicit as usize,
			Depth { implicit } => (!implicit as usize) + 1,
			Sleep { implicit } => !implicit as usize,

			// Interactiv => todo!()e
			Mv { implicit, force: _ }
			| Rm { implicit, force: _ }
			| RmR { implicit, force: _ }
			| Cp { implicit, force: _ }
			| Ln { implicit, force: _ }
			| LnS { implicit, force: _ } => (!implicit as usize) + 1,
			Mkdir => 1,
			Touch { implicit } => !implicit as usize,
		}
	}

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
			("z" | "size" | "filesize", 0 | 1) => implicit!(FileSize),

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
