#![allow(unused)]

/*
 * Structure:
 * IIIIIIII PVoxFFAA OOOOOOOO OOOOOOOO
 * - `I` is a unique ID
 * - `A` is the arity
 * - `F` is the "force mode":
 *    - `00` = use cli default
 *    - `01` = always prompt
 *    - `10` = always force
 *    - `11` = dry mode
 * - `o` is whether it takes an offset
 * - `O` is the offset (unspecified for non-offset opcodes;
 *       all 1s = read next u32 and that's the offset.)
 * - `P` is whether it is taking an implicit path parameter.
 * - `V` is whether it takes a variable amount of arguments
 * - `x` is reserved
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, align(4))]
pub struct Opcode {
	kind: OpKind,
	offset: u16,
}

sa::assert_eq_size!(Opcode, u32);
sa::assert_eq_align!(Opcode, u32);

#[rustfmt::skip]
macro_rules! declop {
	($id:literal $($rest:tt)*) => {
		($id << 8 | declop!(@ $($rest)*)) as u16
	};
	(@) => {0};
	(@ , arity = $arity:literal $($rest:tt)*) => {
		{sa::const_assert!($arity <= Opcode::MAX_ARITY); $arity} | declop!(@ $($rest)*)
	};
	(@ , offset $($rest:tt)*)   => { 0b_0010_00_00 | declop!(@ $($rest)*) };
	(@ , varargs $($rest:tt)*)  => { 0b_0100_00_00 | declop!(@ , offset $($rest)*) };
	(@ , implicit $($rest:tt)*) => { 0b_1000_00_00 | declop!(@ $($rest)*) };
	(@ , ask $($rest:tt)*)      => { ForceMode::Ask as u8 as u32 | declop!(@ $($rest)*) };
	(@ , force $($rest:tt)*)    => { ForceMode::Force as u8 as u32 | declop!(@ $($rest)*) };
	(@ , dry $($rest:tt)*)      => { ForceMode::Dry as u8 as u32 | declop!(@ $($rest)*) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
#[rustfmt::skip]
pub enum OpKind {
	Illegal = declop!(0),

	LoadConstant  = declop!(1, offset),
	LoadArgument  = declop!(2, offset),
	LoadVariable  = declop!(3, offset),
	StoreArgument = declop!(4, offset),
	StoreVariable = declop!(5, offset),

	Dup  = declop!(6, arity = 1),
	Pop  = declop!(7, arity = 1),
	Skip = declop!(8),

	GenericCall = declop!(9, arity = 1, varargs),

	CreatePath   = declop!(10, varargs),
	CreateRegex  = declop!(11, varargs),
	CreateString = declop!(12, varargs),

	Return     = declop!(13, arity = 1),
	Jump       = declop!(14, offset),
	JumpIf     = declop!(15, arity = 1, offset),
	JumpUnless = declop!(16, arity = 1, offset),

	Not           = declop!(20, arity = 1),
	Negate        = declop!(21, arity = 1),
	UPositive     = declop!(22, arity = 1),
	ForcedLogical = declop!(23, arity = 1),

	Add = declop!(30, arity = 2),
	Sub = declop!(31, arity = 2),
	Mul = declop!(32, arity = 2),
	Div = declop!(33, arity = 2),
	Mod = declop!(34, arity = 2),
	Pow = declop!(35, arity = 2),

	Matches    = declop!(40, arity = 2),
	NotMatches = declop!(41, arity = 2),
	Eql        = declop!(42, arity = 2),
	Neq        = declop!(43, arity = 2),
	Lth        = declop!(44, arity = 2),
	Leq        = declop!(45, arity = 2),
	Gth        = declop!(46, arity = 2),
	Geq        = declop!(47, arity = 2),


	// Querying functions
	IsFile        = declop!(50, arity = 1),
	IsFileI       = declop!(50, implicit),
	IsDirectory   = declop!(51, arity = 1),
	IsDirectoryI  = declop!(51, implicit),
	IsSymlink     = declop!(52, arity = 1),
	IsSymlinkI    = declop!(52, implicit),
	IsBinary      = declop!(53, arity = 1),
	IsBinaryI     = declop!(53, implicit),
	IsHidden      = declop!(54, arity = 1),
	IsHiddenI     = declop!(54, implicit),
	IsGitIgnored  = declop!(55, arity = 1),
	IsGitIgnoredI = declop!(55, implicit),
	IsOk          = declop!(56, arity = 1),
	IsOkI         = declop!(56, implicit),


	// Path-related functions
	PushRoot    = declop!(60),
	PushPath    = declop!(61),
	PushPwd     = declop!(62),
	FileSize    = declop!(63, arity = 1),
	FileSizeI   = declop!(63, implicit),
	Dirname     = declop!(64, arity = 1),
	DirnameI    = declop!(64, implicit),
	Extname     = declop!(65, arity = 1),
	ExtnameI    = declop!(65, implicit),
	ExtnameDot  = declop!(66, arity = 1),
	ExtnameDotI = declop!(66, implicit),
	Basename    = declop!(67, arity = 1),
	BasenameI   = declop!(67, implicit),
	Stemname    = declop!(68, arity = 1),
	StemnameI   = declop!(68, implicit),
	Depth       = declop!(69, arity = 1),
	DepthI      = declop!(69, implicit),

	// Misc
	Print  = declop!(80, varargs),
	PrintI = declop!(80, implicit),
	Write  = declop!(81, varargs),
	WriteI = declop!(81, implicit),
	Quit   = declop!(82, arity = 1), // implicit = just push on the stack
	Sleep  = declop!(83, arity = 1), // implicit = just push on stack


	// Interactive
	Mv   = declop!(90, arity = 2),
	MvA  = declop!(90, arity = 2, ask),
	MvF  = declop!(90, arity = 2, force),
	MvD  = declop!(90, arity = 2, dry),
	MvI  = declop!(90, arity = 1, implicit),
	MvIA = declop!(90, arity = 1, implicit, ask),
	MvIF = declop!(90, arity = 1, implicit, force),
	MvID = declop!(90, arity = 1, implicit, dry),

	Rm   = declop!(91, arity = 2),
	RmA  = declop!(91, arity = 2, ask),
	RmF  = declop!(91, arity = 2, force),
	RmD  = declop!(91, arity = 2, dry),
	RmI  = declop!(91, arity = 1, implicit),
	RmIA = declop!(91, arity = 1, implicit, ask),
	RmIF = declop!(91, arity = 1, implicit, force),
	RmID = declop!(91, arity = 1, implicit, dry),

	RmR   = declop!(92, arity = 2),
	RmRA  = declop!(92, arity = 2, ask),
	RmRF  = declop!(92, arity = 2, force),
	RmRD  = declop!(92, arity = 2, dry),
	RmRI  = declop!(92, arity = 1, implicit),
	RmRIA = declop!(92, arity = 1, implicit, ask),
	RmRIF = declop!(92, arity = 1, implicit, force),
	RmRID = declop!(92, arity = 1, implicit, dry),

	Cp   = declop!(93, arity = 2),
	CpA  = declop!(93, arity = 2, ask),
	CpF  = declop!(93, arity = 2, force),
	CpD  = declop!(93, arity = 2, dry),
	CpI  = declop!(93, arity = 1, implicit),
	CpIA = declop!(93, arity = 1, implicit, ask),
	CpIF = declop!(93, arity = 1, implicit, force),
	CpID = declop!(93, arity = 1, implicit, dry),

	Ln   = declop!(94, arity = 2),
	LnA  = declop!(94, arity = 2, ask),
	LnF  = declop!(94, arity = 2, force),
	LnD  = declop!(94, arity = 2, dry),
	LnI  = declop!(94, arity = 1, implicit),
	LnIA = declop!(94, arity = 1, implicit, ask),
	LnIF = declop!(94, arity = 1, implicit, force),
	LnID = declop!(94, arity = 1, implicit, dry),

	LnS   = declop!(95, arity = 2),
	LnSA  = declop!(95, arity = 2, ask),
	LnSF  = declop!(95, arity = 2, force),
	LnSD  = declop!(95, arity = 2, dry),
	LnSI  = declop!(95, arity = 1, implicit),
	LnSIA = declop!(95, arity = 1, implicit, ask),
	LnSIF = declop!(95, arity = 1, implicit, force),
	LnSID = declop!(95, arity = 1, implicit, dry),

	Touch   = declop!(96, arity = 2),
	TouchA  = declop!(96, arity = 2, ask),
	TouchF  = declop!(96, arity = 2, force),
	TouchD  = declop!(96, arity = 2, dry),
	TouchI  = declop!(96, arity = 1, implicit),
	TouchIA = declop!(96, arity = 1, implicit, ask),
	TouchIF = declop!(96, arity = 1, implicit, force),
	TouchID = declop!(96, arity = 1, implicit, dry),

	// Note that `mkdir` doesn't have an implicit verison.
	Mkdir   = declop!(97, arity = 2),
	MkdirA  = declop!(97, arity = 2, ask),
	MkdirF  = declop!(97, arity = 2, force),
	MkdirD  = declop!(97, arity = 2, dry),
}

impl Opcode {
	pub const MAX_ARITY: usize = 3;

	pub const fn new(kind: OpKind, offset: u16) -> Self {
		// either we take an offset or `offset` should be zero.
		assert!(kind.takes_offset() || offset == 0);

		Self { kind, offset }
	}

	pub const fn kind(self) -> OpKind {
		self.kind
	}

	pub const fn offset(self) -> usize {
		self.offset as usize
	}
}

#[repr(u8)]
#[rustfmt::skip]
pub enum ForceMode {
	Default = 0b0000_00_00,
	Ask     = 0b0000_01_00,
	Force   = 0b0000_10_00,
	Dry     = 0b0000_11_00,
}

impl OpKind {
	const fn data(self) -> u8 {
		(self as u32 >> 16) as u8
	}

	pub const fn arity(self) -> usize {
		(self.data() & 0b0000_00_11) as usize
	}

	pub const fn frocemode(self) -> ForceMode {
		unsafe { std::mem::transmute::<u8, ForceMode>(self.data() & 0b0000_11_00) }
	}

	pub const fn takes_offset(self) -> bool {
		self.data() & 0b_0010_0000 != 0
	}

	pub const fn has_implicit_path(self) -> bool {
		self.data() & 0b_1000_0000 != 0
	}

	pub const fn has_va_args(self) -> bool {
		self.data() & 0b_0100_0000 != 0
	}
}
