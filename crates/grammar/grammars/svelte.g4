HTML_CMT : '<!--' ( !'-->' . )* '-->' -> comment ;

unit : script / stuck / styleBlock / node ;

script : '<' 'script' attr* '>' scriptItem* scriptClose -> item ;
scriptClose : '<' '/' 'script' '>' ;
scriptItem : !scriptClose ( item / reactive ) / scriptLoose ;
reactive : '$' ':' ( scope / tail ) -> item ;
scriptLoose : !scriptClose . -> loose ;
stuck : '<' 'script' attr* '>' unread scriptClose? -> item ;
unread : ( !scriptClose . )+ -> loose ;

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
aname : IDENT ( ( ':' / '-' / '|' )+ IDENT )* ;
aval : QUOTE expression QUOTE / STRING / SINGLE / TEMPLATE / template / expression ;

block : ifBlock / scoped ;
scoped : eachBlock / awaitBlock / keyBlock / snippetBlock -> scope ;
ifBlock : LBRACE '#' 'if' cond branch ( 'else' 'if' cond branch )* ( 'else' branch )? 'if' RBRACE ;
cond : probe !!RBRACE / expr* ;
branch : RBRACE node* LBRACE ( ':' / '/' ) -> scope ;
elsePart : LBRACE ':' 'else' RBRACE node* ;

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
openSnippet : LBRACE '#' 'snippet' name fpar? expr* RBRACE ;
closeSnippet : LBRACE '/' 'snippet' RBRACE ;

special : constSpecial / ordinarySpecial ;
constSpecial : LBRACE '@' 'const' name expr* RBRACE -> item ;
ordinarySpecial : LBRACE '@' expr* RBRACE -> item ;

expression : !boundary LBRACE expr* RBRACE ;
boundary : LBRACE ( '#' / ':' / '/' / '@' ) ;
expr : template / arrow / cast / vtarg / bolt / lambda / pgroup / bgroup / group / ( !RBRACE . ) ;
text : !'<' !LBRACE !RBRACE . ;
