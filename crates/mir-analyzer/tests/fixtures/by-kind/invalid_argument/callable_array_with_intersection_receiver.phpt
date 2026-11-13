===description===
`[$obj, 'method']` is callable when `$obj` is an intersection type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Coll { public function add(mixed $e): void; }
interface Sel {}

/**
 * @param Coll&Sel $c
 * @param array<int, object> $xs
 */
function f(object $c, array $xs): void {
    array_walk($xs, [$c, 'add']);
}
