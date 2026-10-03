===description===
array_map with an arrow fn refines the result to array<sourceKey, callbackReturn>
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}
/** @param array<string, Foo> $arr */
function test(array $arr): void {
    $r = array_map(fn(Foo $f): int => 1, $arr);
    /** @mir-check $r is array<string, int> */
    $_ = $r;
}
===expect===
