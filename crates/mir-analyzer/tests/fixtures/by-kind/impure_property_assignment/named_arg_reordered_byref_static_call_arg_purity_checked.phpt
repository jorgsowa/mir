===description===
Same named-argument-reordering bug as the free-function/instance-method
cases, for a static call's by-ref write-back loop (`call/static_call.rs`)
— a by-ref target passed via a named argument out of declared order was
checked against the wrong argument instead of the real by-ref target.
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

class Filler {
    public static function fill(int $skip, array &$out): void {
        $out[] = 1;
    }
}

/** @pure */
function normalize(Bag $b): void {
    Filler::fill(out: $b->items, skip: 1);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method fill() in a pure or immutable context
//                    ^^^^^^^^^ ImpurePropertyAssignment: Assigning to property items of a parameter in a pure or external-mutation-free context
}
===expect===
