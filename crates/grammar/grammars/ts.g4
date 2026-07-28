WS        : [ \t\r\n]+ -> skip ;
LINE_CMT  : '//' ~[\n]* -> comment ;
BLOCK_CMT : '/*' ( !'*/' . )* '*/' -> comment ;
TEMPLATE  : '`' ( '\\' . / ~[`\\] )* '`' ;
STRING    : '"' ( '\\' . / ~["\\] )* '"' ;
SINGLE    : '\'' ( '\\' . / ~['\\] )* '\'' ;
ARROW     : '=>' ;
IDENT     : [a-zA-Z_$] [a-zA-Z0-9_$]* ;
NUMBER    : [0-9] [0-9a-zA-Z_$]* ;
LBRACE    : '{' ;
RBRACE    : '}' ;
LPAREN    : '(' ;
RPAREN    : ')' ;
LBRACK    : '[' ;
RBRACK    : ']' ;
OTHER     : . ;

unit : item ;

item : importItem / exportItem / fnItem / typeItem / interfaceItem / classItem / declItem / ambient / control / callItem / lineItem ;
importItem : 'import' tail -> item ;
exportItem : 'export' ( 'default' )? ( fnItem / typeItem / interfaceItem / classItem / declItem / shipItem / callItem / lineItem ) ;
shipItem : 'type'? LBRACE ( port ( ',' port )* ','? )? RBRACE ( 'from' lit )? ';'? -> item ;
port : 'type'? ( IDENT 'as' name / name ) ;
ambient : 'declare' 'module' lit group ';'? -> item ;
fnItem : flags 'function' name tpar? fpar ( objectReturn / simpleReturn / scope ) -> item ;
tpar : '<' ( tpar / !'>' . )* '>' ;
fpar : LPAREN seat mates RPAREN / pgroup ;
mates : ( ',' more )* frest ;
more : IDENT '?'? ':' stype -> param ;
seat : IDENT '?'? ':' stype -> receiver ;
stype : ( sgroup / ( !',' !RPAREN . ) )* ;
sgroup : pgroup / bgroup / group ;
frest : inner* ;
typeItem : 'type' name tail -> item ;
interfaceItem : 'interface' name head group -> item ;
classItem : 'class' name head group -> item ;
declItem : decl ( name init / bgroup init / group init ) -> item ;
callItem : path tpar? pgroup tail -> item ;
lineItem : line tail -> item ;

name : IDENT -> word ;
flags : ( 'async' )* ;
decl : 'const' / 'let' / 'var' ;
line : 'await' / 'return' / 'throw' / 'break' / 'continue' ;
simpleReturn : ':' plain scope ;
objectReturn : ':' plain group after scope ;
plain : ( !LBRACE !RBRACE !';' . )* ;
after : ( !LBRACE !RBRACE !';' !LPAREN !'function' !'export' !'const' !'let' !'var' !'if' !'for' !'while' !'switch' !'try' !'await' . )* ;
path : mark / IDENT ( '.' IDENT )* ;
mark : 'Deno' '.' 'test' -> test ;

stmt : control / fnItem / typeItem / interfaceItem / classItem / declItem / scope / ( !RBRACE . ) ;
control : ifExpr / whileExpr / forExpr / switchExpr / tryExpr ;
ifExpr : 'if' LPAREN probe RPAREN scope elseTail / 'if' head scope elseTail ;
probe : subject '=' '=' '='? lit ;
subject : IDENT -> probe ;
lit : STRING / SINGLE / TEMPLATE / NUMBER / 'true' / 'false' / 'null' / 'undefined' ;
elseTail : ( 'else' ( ifExpr / scope ) )? ;
whileExpr : 'while' head scope ;
forExpr : 'for' head scope ;
switchExpr : 'switch' head scope ;
tryExpr : 'try' scope catchTail finallyTail ;
catchTail : ( 'catch' head scope )? ;
finallyTail : ( 'finally' scope )? ;

scope : LBRACE stmt* RBRACE -> scope ;

init : ( !ARROW !';' loose )* ( ARROW scope ';'? / ';'? ) ;
tail : ( !';' loose )* ';'? ;

head : headAtom* ;
headAtom : pgroup / bgroup / ( !LBRACE !RBRACE !';' . ) ;
pgroup : LPAREN inner* RPAREN ;
bgroup : LBRACK inner* RBRACK ;
inner : pgroup / bgroup / group / ( !RPAREN !RBRACK . ) ;

loose : group / pgroup / bgroup / ( !LBRACE !RBRACE !LBRACK !RBRACK !LPAREN !RPAREN . ) ;
group : LBRACE loose* RBRACE ;
