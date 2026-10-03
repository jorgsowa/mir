===description===
Forget assertion after relevant non mutation free call
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//                 ^^^^^^^^^ PossiblyNullArgument: Argument $_str of requiresString() might be null
}

function requiresString(string $_str): void {}

===expect===
