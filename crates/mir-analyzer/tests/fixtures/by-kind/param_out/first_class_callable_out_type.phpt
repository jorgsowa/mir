===description===
@param-out type is preserved when a function is captured as a first-class
callable. $id should be int (from @param-out int), not mixed.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param-out int $n
 */
function nextId(mixed &$n): void {
    static $counter = 0;
    $n = ++$counter;
}

$gen = nextId(...);
$gen($id);
/** @mir-check $id is int */
echo $id;
===expect===
