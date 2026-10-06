===description===
Same by-ref-argument gap as the built-in case, but for a plain
user-defined function declaring a `&$param` -- the write-back loop for
user-defined functions is a separate code path from the built-in one and
had the identical Variable-only blind spot.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <ImpureFunctionCall errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    public $n = 0;
}

function bump(int &$n): void {
    $n++;
}

/** @pure */
function run(Box $b): void {
    bump($b->n);
//       ^^^^^ ImpurePropertyAssignment: Assigning to property n of a parameter in a pure or external-mutation-free context
}
