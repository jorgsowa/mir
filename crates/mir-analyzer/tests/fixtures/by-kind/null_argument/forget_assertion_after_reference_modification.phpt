===description===
Forget assertion after reference modification
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class Foo
{
    public ?string $bar = null;
}

/**
 * @assert-if-true !null $foo->bar
 */
function assertBarNotNull(Foo $foo): bool
{
    return $foo->bar !== null;
}

$foo = new Foo();
$barRef = &$foo->bar;

if (assertBarNotNull($foo)) {
    $barRef = null;
    requiresString($foo->bar);
}

function requiresString(string $_str): void {}

===expect===
PossiblyNullArgument@20:19-20:28: Argument $_str of requiresString() might be null
