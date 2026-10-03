===description===
Negative control: an explicit `@var` docblock still takes priority over both the native
hint and literal narrowing — a docblock that deliberately widens back to plain `int`
must still be honored, discarding the literal's own positive-int precision.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Foo {
    /** @var int */
    private const int ID = 5;
    public function bar(): void {
        baz([self::ID]);
//          ^^^^^^^^^^ InvalidArgument: Argument $ids of baz() expects 'list<positive-int>', got 'array{0: int}'
    }
}

/** @param list<positive-int> $ids */
function baz(array $ids): void {}

===expect===
