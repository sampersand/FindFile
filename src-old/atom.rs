struct Expression;

enum Atom {
	Expr(Expression),
	Variable(Variable),
}

atom
 := '(' expression ')'
  | variable  | dollar-variable
  | shell-function
  | string
  | number
  | filesize
  | time | date
  | pcre-regex | posix-regex
  ;
