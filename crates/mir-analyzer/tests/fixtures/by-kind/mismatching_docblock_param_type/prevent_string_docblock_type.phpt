===description===
Prevent string docblock type
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param string $mapper
 */
function map2(callable $mapper): void {}

map2("foo");
