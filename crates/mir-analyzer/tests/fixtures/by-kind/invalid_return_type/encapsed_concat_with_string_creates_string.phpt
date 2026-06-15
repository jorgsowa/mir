===description===
Encapsed concat with string creates string
===config===
suppress=ImplicitToStringCast
===file===
<?php
/**
 * @param  literal-string $s2
 * @return literal-string
 */
function foo(string $s1, string $s2): string {
    return "hello $s1 $s2";
}
===expect===
UndefinedDocblockClass@6:9-6:12: Docblock type 'literal-string' does not exist
