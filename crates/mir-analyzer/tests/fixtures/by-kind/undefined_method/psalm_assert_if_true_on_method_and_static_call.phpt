===description===
W4: `@psalm-assert-if-true` inside an `if` condition on an instance-method
call and a static call now narrows — the conditional dispatcher only ever
resolved a free function (via `find_function`), never a method/static
call, so the same docblock form G6 already fixed for bare statements was
still silently ignored inside a condition. Also covers a method-level
`@template T` substituting into the assertion's type.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class User {
    public function name(): string { return ''; }
}

class Dog {}

class Checker {
    /** @psalm-assert-if-true User $value */
    public function isUserInstance(mixed $value): bool { return $value instanceof User; }

    /** @psalm-assert-if-true User $value */
    public static function isUserStatic(mixed $value): bool { return $value instanceof User; }

    /**
     * @template T of object
     * @param T $seed
     * @psalm-assert-if-true T $target
     */
    public function isSameType(object $seed, mixed $target): bool { return true; }
}

function test_instance_call(Checker $c, mixed $value): void {
    if ($c->isUserInstance($value)) {
        $value->name();
        $value->missing();
    }
}

function test_static_call(mixed $value): void {
    if (Checker::isUserStatic($value)) {
        $value->name();
        $value->missing();
    }
}

function test_template_bearing(Checker $c, Dog $seed, mixed $target): void {
    if ($c->isSameType($seed, $target)) {
        /** @mir-check $target is Dog */
        $target;
    }
}

function test_unguarded_stays_mixed(Checker $c, mixed $value): void {
    // No `if` guard — assertion never applies, $value stays mixed.
    $c->isUserInstance($value);
    $value->missing();
}
===expect===
UndefinedMethod@26:8-26:25: Method User::missing() does not exist
UndefinedMethod@33:8-33:25: Method User::missing() does not exist
MixedMethodCall@47:4-47:21: Method missing() called on mixed type
