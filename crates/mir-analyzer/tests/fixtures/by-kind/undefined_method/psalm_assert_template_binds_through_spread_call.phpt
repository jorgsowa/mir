===description===
`compute_assertion_template_bindings` never expanded a literal spread call
argument before computing `arg_types` — a `@template T` meant to be
inferred from a sibling `class-string<T>` positional argument silently
defaulted instead of binding to the concrete class, so the
`@psalm-assert-if-true T $value` narrowing that should apply never did.
===config===
suppress=MixedAssignment
===file===
<?php
class Foo {
    public function onlyOnFoo(): void {}
}


/**
 * @template T
 * @param class-string<T> $class
 * @psalm-assert-if-true T $value
 */
function isInstanceOf(mixed $value, string $class): bool {
    return $value instanceof $class;
}

function test(mixed $x): void {
    if (isInstanceOf(...[$x, Foo::class])) {
        $x->missing();
    }
}
===expect===
UndefinedMethod@18:8-18:21: Method Foo::missing() does not exist
