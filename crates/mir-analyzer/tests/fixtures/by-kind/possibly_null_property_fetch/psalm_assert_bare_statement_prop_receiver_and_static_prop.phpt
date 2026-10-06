===description===
Bare-statement `@psalm-assert` (outside any `if`/`while`, handled by
call/function.rs, a separate implementation from narrowing.rs's
apply_docblock_assertions) narrows a property receiver non-null too, and
now also supports a static-property argument — parity fixes matching R1-1.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//              ^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $child on possibly null value
    $h->child->foo();
//  ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Bar::foo() does not exist
}

function narrowsStaticProp(): void {
    assertIsBar(Holder::$staticChild);
    Holder::$staticChild->foo();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Bar::foo() does not exist
}
