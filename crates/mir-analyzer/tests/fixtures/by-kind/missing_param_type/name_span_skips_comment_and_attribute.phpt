===description===
Issues on a declaration name point at the name token, not at an earlier
occurrence of the same text in a comment or an attribute argument.
===file===
<?php
function a /* a */ () { return 1; }
//       ^ MissingReturnType: Function a() has no return type annotation

function f(#[\SensitiveParameter('$x')] $x): void { echo $x; }
//                                      ^^ MissingParamType: Parameter $x of f() has no type annotation
