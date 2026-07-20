===description===
Bare-statement `@psalm-assert` (outside any `if`/`while`, handled by
call/function.rs, a separate implementation from narrowing.rs's
apply_docblock_assertions) narrows a property receiver non-null too, and
now also supports a static-property argument — parity fixes matching R1-1.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,MissingConstructor
===file===
<?php
class Bar {}

final class Holder {
    /** @var Bar|null */
    public $child;

    /** @var Bar|null */
    public static $staticChild;
}

/** @psalm-assert Bar $x */
function assertIsBar(mixed $x): void {}

function narrowsPropReceiver(?Holder $h): void {
    assertIsBar($h->child);
    $h->child->foo();
}

function narrowsStaticProp(): void {
    assertIsBar(Holder::$staticChild);
    Holder::$staticChild->foo();
}
===expect===
PossiblyNullPropertyFetch@16:16-16:25: Cannot access property $child on possibly null value
UndefinedMethod@17:4-17:20: Method Bar::foo() does not exist
UndefinedMethod@22:4-22:31: Method Bar::foo() does not exist
