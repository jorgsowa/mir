===description===
A bare-statement (unconditional) `@psalm-assert Type $arr['key']` call
never applied the array-key-targeted narrowing at all -- function.rs,
method.rs, and static_call.rs each hand-duplicated their own hardcoded
var/prop/static-prop application loop for this call shape, none of which
ever read `assertion.param_key`. Now routed through the same shared
apply_one_assertion the conditional if-true/if-false dispatch uses.
===config===
suppress=MissingReturnType,MixedArgument,UnusedParam
===file===
<?php
/** @psalm-assert string $arr['key'] */
function assertHasKeyFn(array $arr): void {}

/** @param array{key?: string} $c */
function testFreeFunction(array $c): void {
    assertHasKeyFn($c);
    /** @mir-check $c is array{key: string} */
    $_ = 1;
}

class Validator {
    /** @psalm-assert string $arr['key'] */
    public function assertHasKeyMethod(array $arr): void {}

    /** @psalm-assert string $arr['key'] */
    public static function assertHasKeyStatic(array $arr): void {}
}

/** @param array{key?: string} $c */
function testMethod(Validator $v, array $c): void {
    $v->assertHasKeyMethod($c);
    /** @mir-check $c is array{key: string} */
    $_ = 1;
}

/** @param array{key?: string} $c */
function testStaticMethod(array $c): void {
    Validator::assertHasKeyStatic($c);
    /** @mir-check $c is array{key: string} */
    $_ = 1;
}
===expect===
