===description===
Literal concat with string creates string
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
