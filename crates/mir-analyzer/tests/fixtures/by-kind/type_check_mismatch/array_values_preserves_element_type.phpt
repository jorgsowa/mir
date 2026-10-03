===description===
array_values returns a list preserving the source value type
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
    $vals = array_values($arr);
    /** @mir-check $vals is list<Foo> */
    $_ = $vals;
}
===expect===
