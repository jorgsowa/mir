===description===
Numeric string is not non falsy
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
