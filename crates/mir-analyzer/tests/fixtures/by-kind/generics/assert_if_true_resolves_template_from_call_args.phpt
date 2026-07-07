===description===
FP: a `@psalm-assert-if-true T $value`-style generic type guard (T bound via
a sibling `class-string<T>` argument) narrowed the asserted variable to the
bare, unresolved template atom `T` instead of the concrete class the call
actually proved — so a later `@mir-check`/property access against the real
type mismatched. The plain (non-generic) `@psalm-assert` path used at direct
call sites already substituted template bindings before narrowing; the
narrowing-time assert-if-true/-if-false path did not.
===config===
suppress=UnusedParameter
===file===
<?php
/**
 * @template T of object
 * @param mixed $value
 * @param class-string<T> $class
 * @psalm-assert-if-true T $value
 */
function isInstanceOf($value, string $class): bool {
    return $value instanceof $class;
}

class Animal {}
class Dog extends Animal {}

function test(mixed $value): void {
    if (isInstanceOf($value, Dog::class)) {
        /** @mir-check $value is Dog */
        echo "ok";
    }
}
===expect===
