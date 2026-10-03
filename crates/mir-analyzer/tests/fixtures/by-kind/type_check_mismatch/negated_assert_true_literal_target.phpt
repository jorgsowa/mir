===description===
`negate_assertion_type` had arms for `TNull`/`TFalse`/named-object atoms
but none for `TTrue` — `@psalm-assert !true $v` on a `bool` fell to the
catch-all and left the type unchanged instead of narrowing to `false`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param mixed $v
 * @psalm-assert !true $v
 */
function assertNotTrue($v): void {}

function test(bool $b): void {
    assertNotTrue($b);
    /** @mir-check $b is false */
    echo "ok";
}
===expect===
