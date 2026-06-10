===description===
Concatenate negative int right side is not numeric
===file===
<?php
/**
 * @param numeric-string $bar
 * @return int
 */
function foo(string $bar): int
{
    return (int) $bar;
}

foo(foo("123") . foo("-456"));

===expect===
