===description===
Literal concat with string creates string. Previously baked in the
`literal-string` docblock-keyword bug: expected `UndefinedDocblockClass`
since the keyword wasn't recognized at all.
===config===
suppress=ImplicitToStringCast
===file===
<?php
/**
 * @param  literal-string $s2
 * @return literal-string
 */
function foo(string $s1, string $s2): string {
    return $s1 . $s2;
}
===expect===
