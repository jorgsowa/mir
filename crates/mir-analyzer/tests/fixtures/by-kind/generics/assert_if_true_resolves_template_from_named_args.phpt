===description===
FP: same as assert_if_true_resolves_template_from_call_args, but the call site
passes its arguments as named arguments in reversed textual order. The
assert-if-true template binder built its own positional arg list via
`call.args.get(i)`, assuming argument i always fed parameter i — a named
argument out of declared order fed the wrong parameter's value into T's
inference, resolving T to the wrong (or unresolved) type.
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
    if (isInstanceOf(class: Dog::class, value: $value)) {
        /** @mir-check $value is Dog */
        echo "ok";
    }
}
===expect===
