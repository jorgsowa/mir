===description===
Template-binding inference for an assert-if-true call resolves a template
from a property-access argument, not just a bare variable — the template
here (`T` from `$this->seedProp`) is decoupled from the narrowed target
(`$target`, a plain variable) so only assertion_arg_type's own
property-access gap is under test.
===config===
suppress=UnusedParameter,MissingConstructor
===file===
<?php
/**
 * @template T of object
 * @param T $seed
 * @psalm-assert-if-true T $target
 */
function matchesSeed($seed, mixed $target): bool {
    return $seed == $target;
}

class Animal {}
class Dog extends Animal {}

final class Holder {
    public Dog $seedProp;

    public function test(mixed $target): void {
        if (matchesSeed($this->seedProp, $target)) {
            /** @mir-check $target is Dog */
            echo "ok";
        }
    }
}
===expect===
