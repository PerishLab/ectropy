HTML_CMT : '<!--' ( !'-->' . )* '-->' -> comment ;

unit : script / styleBlock / node ;

script : '<' 'script' attr* '>' scriptItem* scriptClose -> item ;
scriptClose : '<' '/' 'script' '>' ;
scriptItem : item / reactive / scriptLoose ;
reactive : '$' ':' tail -> item ;
scriptLoose : !scriptClose . -> loose ;

styleBlock : '<' 'style' attr* '>' styleToken* styleClose -> style ;
styleClose : '<' '/' 'style' '>' ;
styleToken : !styleClose . ;

node : fragment / lone / pair / block / special / expression / text ;
fragment : '<' '>' node* '<' '/' '>' -> markup ;
lone : '<' tag attr* '/' '>' -> markup ;
pair : '<' tag attr* '>' node* closer -> markup ;
closer : '<' '/' tag? '>' ;
tag : IDENT ( ':' IDENT )? ;

attr : named / expression ;
named : aname ( '=' aval )? ;
aname : IDENT ( ( ':' / '-' / '|' ) IDENT )* ;
aval : STRING / SINGLE / TEMPLATE / expression ;

block : ifBlock / eachBlock / awaitBlock / keyBlock / snippetBlock -> scope ;
ifBlock : openIf node* elseIf* elsePart? closeIf ;
openIf : LBRACE '#' 'if' expr* RBRACE ;
elseIf : LBRACE ':' 'else' 'if' expr* RBRACE node* ;
elsePart : LBRACE ':' 'else' RBRACE node* ;
closeIf : LBRACE '/' 'if' RBRACE ;

eachBlock : openEach node* elsePart? closeEach ;
openEach : LBRACE '#' 'each' expr* RBRACE ;
closeEach : LBRACE '/' 'each' RBRACE ;

awaitBlock : openAwait node* thenPart? catchPart? closeAwait ;
openAwait : LBRACE '#' 'await' expr* RBRACE ;
thenPart : LBRACE ':' 'then' expr* RBRACE node* ;
catchPart : LBRACE ':' 'catch' expr* RBRACE node* ;
closeAwait : LBRACE '/' 'await' RBRACE ;

keyBlock : openKey node* closeKey ;
openKey : LBRACE '#' 'key' expr* RBRACE ;
closeKey : LBRACE '/' 'key' RBRACE ;

snippetBlock : openSnippet node* closeSnippet ;
openSnippet : LBRACE '#' 'snippet' name expr* RBRACE ;
closeSnippet : LBRACE '/' 'snippet' RBRACE ;

special : constSpecial / ordinarySpecial ;
constSpecial : LBRACE '@' 'const' name expr* RBRACE -> item ;
ordinarySpecial : LBRACE '@' expr* RBRACE -> item ;

expression : !boundary LBRACE expr* RBRACE ;
boundary : LBRACE ( '#' / ':' / '/' / '@' ) ;
expr : pgroup / bgroup / group / ( !RBRACE . ) ;
text : !'<' !LBRACE !RBRACE . ;
