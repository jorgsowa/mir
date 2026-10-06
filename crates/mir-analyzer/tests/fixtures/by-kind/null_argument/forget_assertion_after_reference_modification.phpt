===description===
Forget assertion after reference modification
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//                 ^^^^^^^^^ PossiblyNullArgument: Argument $_str of requiresString() might be null
}

function requiresString(string $_str): void {}
