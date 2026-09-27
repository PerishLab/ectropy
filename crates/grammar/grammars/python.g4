WS       : [ \t\r\n]+ -> skip ;
JOIN     : '\\' '\r'? '\n' -> skip ;
COMMENT  : '#' ~[\n]* -> comment ;
TRIPLE2  : [rRbBuUfF]* '"' '"' '"' ( !('"' '"' '"') . )* '"' '"' '"' ;
TRIPLE1  : [rRbBuUfF]* '\'' '\'' '\'' ( !('\'' '\'' '\'') . )* '\'' '\'' '\'' ;
STRING2  : [rRbBuUfF]* '"' ( '\\' . / ~["\\\n] )* '"' ;
STRING1  : [rRbBuUfF]* '\'' ( '\\' . / ~['\\\n] )* '\'' ;
IDENT    : [a-zA-Z_] [a-zA-Z0-9_]* ;
NUMBER   : [0-9] [0-9a-zA-Z_\.]* ;
LPAREN   : '(' ;
RPAREN   : ')' ;
LBRACK   : '[' ;
RBRACK   : ']' ;
LBRACE   : '{' ;
RBRACE   : '}' ;
OP       : [+/\-*@%&|^~<>=:.,;!] ;
OTHER    : . ;

unit : stmt ;

stmt : decorated / fnItem / classItem / ifStmt / whileStmt / forStmt / withStmt / tryStmt / matchStmt / caseStmt / simple ;

decorated : decorator+ ( fnItem / classItem ) ;
decorator : '@' rest NEWLINE ;

fnItem : 'async'? 'def' name fparams returns? ':' suite -> item ;
classItem : 'class' name pgroup? ':' suite -> item ;
returns : '-' '>' header ;

fparams : LPAREN RPAREN / LPAREN seat tails RPAREN / LPAREN marker ( ',' entry )* ','? RPAREN ;
tails : ( ',' entry )* ','? ;
entry : more / marker ;
marker : '/' / '*' ;
seat : parameter -> receiver ;
more : parameter -> param ;
parameter : stars? name hint? preset? ;
stars : '*' '*'? ;
hint : ':' value ;
preset : '=' value ;
value : valueAtom* ;
valueAtom : pgroup / bgroup / dgroup / !',' !RPAREN scalar ;

ifStmt : 'if' header ':' suite elifPart* elsePart? ;
elifPart : 'elif' header ':' suite ;
elsePart : 'else' ':' suite ;
whileStmt : 'while' header ':' suite elsePart? ;
forStmt : 'async'? 'for' header ':' suite elsePart? ;
withStmt : 'async'? 'with' header ':' suite ;
tryStmt : 'try' ':' suite exceptPart+ elsePart? finallyPart? / 'try' ':' suite finallyPart ;
exceptPart : 'except' header ':' suite / 'except' ':' suite ;
finallyPart : 'finally' ':' suite ;
matchStmt : 'match' header ':' suite ;
caseStmt : 'case' header ':' suite ;

suite : scope / inline ;
scope : NEWLINE INDENT stmt+ DEDENT -> scope ;
inline : rest NEWLINE -> scope ;

simple : importItem / assignItem / flowItem / plainItem ;
importItem : ( 'from' path 'import' ports / 'import' ports ) NEWLINE -> item ;
ports : pgroup / port ( ',' port )* ','? ;
port : path ( 'as' name )? ;
path : IDENT ( '.' IDENT )* ;

assignItem : target assign rest NEWLINE -> item ;
target : name / targetGroup ;
targetGroup : LPAREN targets RPAREN / LBRACK targets RBRACK / targets ;
targets : name ( ',' name )+ ','? ;
assign : '=' / ':' / '+' '=' / '-' '=' / '*' '=' / '/' '=' / '|' '=' / '&' '=' / '^' '=' / '%' '=' ;

flowItem : flow rest NEWLINE -> item ;
flow : 'return' / 'raise' / 'yield' / 'assert' / 'del' / 'pass' / 'break' / 'continue' / 'global' / 'nonlocal' ;
plainItem : !'async' !'def' !'class' !'if' !'elif' !'else' !'while' !'for' !'with' !'try' !'except' !'finally' !'match' !'case' !'@' rest NEWLINE -> item ;

name : IDENT -> word ;
header : headerAtom* ;
headerAtom : pgroup / bgroup / dgroup / !':' scalar ;
rest : part+ ;
part : pgroup / bgroup / dgroup / scalar ;
pgroup : LPAREN part* RPAREN ;
bgroup : LBRACK part* RBRACK ;
dgroup : LBRACE part* RBRACE -> literal ;
scalar : IDENT / NUMBER / TRIPLE2 / TRIPLE1 / STRING2 / STRING1 / OP ;
