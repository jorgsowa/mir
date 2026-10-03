===description===
Negative control for the typed-class-const literal-narrowing fix: narrowing to the
literal value must not become a blanket bypass — a literal that genuinely violates the
target type (a negative int against `positive-int`) still flags.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Foo {
    private const int ID = -5;
    public function bar(): void {
        baz([self::ID]);
//          ^^^^^^^^^^ InvalidArgument: Argument $ids of baz() expects 'list<positive-int>', got 'array{0: -5}'
    }
}

/** @param list<positive-int> $ids */
function baz(array $ids): void {}

===expect===
