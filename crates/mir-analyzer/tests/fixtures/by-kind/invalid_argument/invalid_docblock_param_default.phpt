===description===
Invalid docblock param default
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param  int $p
 * @return void
 */
function f($p = false) {}
===expect===
