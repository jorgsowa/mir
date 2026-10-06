===description===
foreach over an array_map result binds the loop variable to the callback return type
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
class Bar {}
/** @param list<Foo> $items */
function test(array $items): void {
    $mapped = array_map(fn(Foo $f): Bar => new Bar(), $items);
    foreach ($mapped as $m) {
        /** @mir-check $m is Bar */
        $_ = $m;
    }
}
