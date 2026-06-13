===description===
Forget assertion after relevant non mutation free call on reference
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
UnsupportedReferenceUsage@21:1-21:16: Reference assignment is not supported
PossiblyNullArgument@25:20-25:29: Argument $_str of requiresString() might be null
