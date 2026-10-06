===description===
Negative control: when the enum-typed receiver itself is nullable,
covering every case's `->value` still isn't exhaustive without an
explicit `null` arm — `$type->value` on a null `$type` evaluates to null.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Kind: string {
    case Foo = 'foo';
    case Bar = 'bar';
}
function h(?Kind $type): bool {
    return match ($type->value) {
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: null
//                ^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $value on possibly null value
        Kind::Foo->value => true,
        Kind::Bar->value => false,
    };
}
