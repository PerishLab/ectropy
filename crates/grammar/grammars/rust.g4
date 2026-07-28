WS        : [ \t\r\n]+ -> skip ;
LINE_CMT  : '//' ~[\n]* -> comment ;
BLOCK_CMT : '/*' ( !'*/' . )* '*/' -> comment ;
BRAW1     : 'br#"' ( !'"#' . )* '"#' ;
RAW1      : 'r#"' ( !'"#' . )* '"#' ;
STRING    : '"' ( '\\' . / ~["\\] )* '"' ;
CHAR      : '\'' ( '\\' . / ~['\\] ) '\'' ;
IDENT     : [a-zA-Z_] [a-zA-Z0-9_]* ;
NUMBER    : [0-9] [0-9a-zA-Z_]* ;
LBRACE    : '{' ;
RBRACE    : '}' ;
LPAREN    : '(' ;
RPAREN    : ')' ;
LBRACK    : '[' ;
RBRACK    : ']' ;
OTHER     : . ;

unit : item ;

item : outer* vis core ;
outer : sign / plain ;
sign : '#' '!'? LBRACK gate RBRACK -> test ;
gate : 'cfg' LPAREN 'test' RPAREN / 'test' ;
plain : '#' '!'? LBRACK loose* RBRACK ;
vis : ( 'pub' pgroup? )? ;
core : modItem / nestItem / fnItem / structItem / makeItem / callItem / lineItem ;

modItem    : quals 'impl' head LBRACE unit* RBRACE -> item ;
nestItem   : quals ( 'trait' / 'mod' ) name head LBRACE unit* RBRACE -> item ;
fnItem     : quals 'fn' name fnTail -> item ;
fnTail : gen? fparams head ( scope / ';' ) / head ( scope / ';' ) ;
gen : '<' genAtom* '>' ;
genAtom : gen / pgroup / bgroup / ( !'>' !LPAREN . ) ;
fparams : LPAREN seat mates RPAREN / LPAREN sits mates RPAREN ;
sits : '&'? 'mut'? 'self' ;
mates : ( ',' more )* frest ;
more : 'mut'? IDENT ':' stype -> param ;
seat : 'mut'? IDENT ':' stype -> receiver ;
stype : ( sgroup / ( !',' !RPAREN . ) )* ;
sgroup : pgroup / bgroup / gen ;
frest : inner* ;
structItem : quals ( 'struct' / 'enum' / 'union' ) name head structTail -> item ;
lineItem   : declItem / plainItem ;
declItem : declKw 'mut'? name ( !';' loose )* ';'? -> item ;
plainItem : lineKw ( !';' loose )* ';'? -> item ;
declKw : 'static' / 'type' / 'const' / 'mod' ;
name       : IDENT -> word ;
quals      : ( 'async' / 'unsafe' / 'const' / 'extern' / 'default' / 'move' )* ;
lineKw     : 'use' / 'extern' ;

makeItem : 'macro_rules' '!' name LBRACE arm* RBRACE -> item ;
arm : match '=' '>' spell ';'? ;
match : pgroup / bgroup / hold ;
spell : scope / pgroup / bgroup ;
hold : LBRACE loose* RBRACE ;
callItem : path '!' ( pgroup / bgroup / hold ) ';'? -> item ;

structTail : fields / ';' ;
fields : LBRACE member* RBRACE ;
member : outer* vis? name membertail ;
membertail : ( !',' !RBRACE loose )* ','? ;

stmt : control / letStmt / structLit / scope / ( !RBRACE . ) ;
letStmt : 'let' 'mut'? name !!( '=' / ':' !':' / ';' ) ;
control : ifExpr / whileExpr / forExpr / loopExpr / matchExpr ;
ifExpr    : 'if' probe scope elseTail / 'if' head scope elseTail ;
probe : subject '=' '=' lit ;
subject : IDENT -> probe ;
lit : STRING / CHAR / RAW1 / BRAW1 / 'b' CHAR / 'b' STRING / NUMBER / 'true' / 'false' ;
elseTail  : ( 'else' ( ifExpr / scope ) )? ;
whileExpr : 'while' head scope ;
forExpr   : 'for' head scope ;
loopExpr  : 'loop' scope ;
matchExpr : 'match' head scope ;

scope : LBRACE stmt* RBRACE -> scope ;
structLit : path LBRACE loose* RBRACE -> literal ;
path : IDENT ( ':' ':' IDENT )* ;

head : headAtom* ;
headAtom : pgroup / bgroup / ( !LBRACE !RBRACE !';' . ) ;
pgroup : LPAREN inner* RPAREN ;
bgroup : LBRACK inner* RBRACK ;
inner  : pgroup / bgroup / scope / structLit / ( !RPAREN !RBRACK . ) ;

loose : group / ( !LBRACE !RBRACE !LBRACK !RBRACK !LPAREN !RPAREN . ) ;
group : LBRACE loose* RBRACE / LBRACK loose* RBRACK / LPAREN loose* RPAREN ;
