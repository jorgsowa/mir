===description===
Invalid docblock for bad annotation
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param-out array<a(),bool> $ar
 */
function foo(array &$ar) : void {}
===expect===
