===description===
Bare-statement `@psalm-assert` on a NULLSAFE property-access argument
never narrowed at all, across all 3 bare-statement call shapes (free
function, method, static method) -- each has its own near-identical
extract_prop_access call site (call/function.rs, call/method.rs,
call/static_call.rs), none of which accepted the nullsafe (`?->`) form.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,MissingConstructor
===file===
<?php
class Bar {}

final class Holder {
    /** @var Bar|null */
    public $child;

    /** @psalm-assert Bar $x */
    public function assertMethod(mixed $x): void {}

    /** @psalm-assert Bar $x */
    public static function assertStatic(mixed $x): void {}
}

/** @psalm-assert Bar $x */
function assertIsBar(mixed $x): void {}

function narrowsViaFreeFunction(?Holder $h): void {
    assertIsBar($h?->child);
    $h->child->foo();
}

function narrowsViaMethod(?Holder $h, Holder $asserter): void {
    $asserter->assertMethod($h?->child);
    $h->child->foo();
}

function narrowsViaStaticMethod(?Holder $h): void {
    Holder::assertStatic($h?->child);
    $h->child->foo();
}
===expect===
UndefinedMethod@20:4-20:20: Method Bar::foo() does not exist
UndefinedMethod@25:4-25:20: Method Bar::foo() does not exist
UndefinedMethod@30:4-30:20: Method Bar::foo() does not exist
