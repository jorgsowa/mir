===description===
Tagged-union match arms reject keys from other variants.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{type: 'a', foo: int}|array{type: 'b', bar: string} $x */
function f(array $x): void {
    match ($x['type']) {
        'a' => $x['bar'],
//                ^^^^^ NonExistentArrayOffset: Array offset 'bar' does not exist
        'b' => $x['bar'],
    };
}
===expect===
