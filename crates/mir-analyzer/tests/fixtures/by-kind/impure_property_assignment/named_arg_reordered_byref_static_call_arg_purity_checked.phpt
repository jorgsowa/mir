===description===
Same named-argument-reordering bug as the free-function/instance-method
cases, for a static call's by-ref write-back loop (`call/static_call.rs`)
— a by-ref target passed via a named argument out of declared order was
checked against the wrong argument instead of the real by-ref target.
===config===
suppress=MissingConstructor,MixedArgument,MixedArrayAssignment,UnusedParam,ImpureFunctionCall
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
}
===expect===
ImpureMethodCall@14:4-14:41: Calling impure method fill() in a pure or immutable context
ImpurePropertyAssignment@14:22-14:31: Assigning to property items of a parameter in a pure or external-mutation-free context
