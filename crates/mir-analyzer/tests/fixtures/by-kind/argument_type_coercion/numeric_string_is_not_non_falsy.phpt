===description===
Numeric string is not non falsy
===ignore===
TODO
===file===
<?php
/** @param non-falsy-string $arg */
function foo(string $arg): string
{
    return $arg;
}

/** @return numeric-string */
function bar(): string
{
    return "0";
}

foo(bar());

===expect===
InvalidReturnType@11:5-11:16: Return type '"0"' is not compatible with declared 'numeric-string'
