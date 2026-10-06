===description===
The by-ref purity write-back loop indexed `call.args` by the parameter's
DECLARED position, ignoring that a named argument can reorder the
call-site's textual position — a named-arg-reordered by-ref target
(`out` here is declared second but passed first) was checked against the
wrong argument (`$b`, a plain non-byref param, which is a no-op) instead
of the real by-ref target (`$b->items`).
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAssignment errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <ImpureFunctionCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Bag {
    public array $items = [];
}

function fill(Bag $skip, array &$out): void {
    $out[] = 1;
}

/** @pure */
function normalize(Bag $b): void {
    fill(out: $b->items, skip: $b);
//            ^^^^^^^^^ ImpurePropertyAssignment: Assigning to property items of a parameter in a pure or external-mutation-free context
}
