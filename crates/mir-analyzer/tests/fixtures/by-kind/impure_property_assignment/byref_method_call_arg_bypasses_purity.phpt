===description===
Same by-ref-argument gap again, but through a method call -- method calls'
by-ref write-back only ever ran for a param declaring `@param-out`, and
even then only matched `ExprKind::Variable`, so a property argument was
never checked regardless.
===config===
suppress=MissingPropertyType,MissingConstructor,MixedArgument
===file===
<?php
class Box {
    public $n = 0;
}

class Helper {
    public function bump(int &$n): void {
        $n++;
    }
}

/** @pure */
function run(Box $b, Helper $h): void {
    $h->bump($b->n);
}
===expect===
ImpureMethodCall@14:4-14:19: Calling impure method bump() in a pure or immutable context
ImpurePropertyAssignment@14:13-14:18: Assigning to property n of a parameter in a pure or external-mutation-free context
