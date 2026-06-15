===description===
Forget assertion after relevant non mutation free call on reference
===config===
suppress=UnusedParam
===file===
<?php
class Foo
{
    public ?string $bar = null;

    public function nonMutationFree(): void
    {
        $this->bar = null;
    }
}

/**
 * @assert-if-true !null $foo->bar
 */
function assertBarNotNull(Foo $foo): bool
{
    return $foo->bar !== null;
}

$foo = new Foo();
$fooRef = &$foo;

if (assertBarNotNull($foo)) {
    $fooRef->nonMutationFree();
    requiresString($foo->bar);
}

function requiresString(string $_str): void {}

===expect===
UnsupportedReferenceUsage@21:0-21:15: Reference assignment is not supported
PossiblyNullArgument@25:19-25:28: Argument $_str of requiresString() might be null
