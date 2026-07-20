===description===
G6: `@psalm-assert` on a bare-statement instance-method call
(`$v->assertUser($value)`) and static call (`Validator::assertUser($value)`)
narrows the argument — only free functions dispatched assertions before;
`ResolvedMethod` had no `assertions` field at all, so method/static calls
silently ignored every `@psalm-assert` docblock. Also covers a
method-level `@template T` substituting into the assertion's type.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class User {
    public function name(): string { return ''; }
}

class Dog {}

class Validator {
    /** @psalm-assert User $value */
    public function assertUserInstance(mixed $value): void {}

    /** @psalm-assert User $value */
    public static function assertUserStatic(mixed $value): void {}

    /**
     * @template T of object
     * @param T $seed
     * @psalm-assert T $target
     */
    public function assertSameType(object $seed, mixed $target): void {}
}

function test_instance_call(Validator $v, mixed $value): void {
    $v->assertUserInstance($value);
    $value->name();
    $value->missing();
}

function test_static_call(mixed $value): void {
    Validator::assertUserStatic($value);
    $value->name();
    $value->missing();
}

function test_template_bearing(Validator $v, Dog $seed, mixed $target): void {
    $v->assertSameType($seed, $target);
    /** @mir-check $target is Dog */
    $target;
}
===expect===
UndefinedMethod@26:4-26:21: Method User::missing() does not exist
UndefinedMethod@32:4-32:21: Method User::missing() does not exist
