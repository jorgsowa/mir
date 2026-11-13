===description===
Psalm's `UndefinedPropertyFetch` and `UndefinedPropertyAssignment` suppress mir's
`UndefinedProperty` on a read and a write.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Msg { public int $a = 0; }

function f(Msg $msg): int {
    /** @psalm-suppress UndefinedPropertyFetch */
    return $msg->nope;
}


final class Box {
    public function set(): void {
        /** @psalm-suppress UndefinedPropertyAssignment */
        self::$missing = 1;
    }
}
