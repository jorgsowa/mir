===description===
Forget assertion after relevant non mutation free call
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

if (assertBarNotNull($foo)) {
    $foo->nonMutationFree();
    requiresString($foo->bar);
}

function requiresString(string $_str): void {}

===expect===
PossiblyNullArgument
===ignore===
TODO
