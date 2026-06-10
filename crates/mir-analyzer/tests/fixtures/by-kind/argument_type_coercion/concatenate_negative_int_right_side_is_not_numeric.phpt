===description===
Concatenate negative int right side is not numeric
===ignore===
TODO
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
InvalidArgument@11:9-11:14: Argument $bar of foo() expects 'numeric-string', got '"123"'
InvalidArgument@11:22-11:28: Argument $bar of foo() expects 'numeric-string', got '"-456"'
