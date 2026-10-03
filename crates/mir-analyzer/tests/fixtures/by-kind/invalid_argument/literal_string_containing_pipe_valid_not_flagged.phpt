===description===
A valid literal-string argument containing '|' matching its docblock union
member exactly is not flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 'a|b'|'c' $x */
function g($x): void {}

g('a|b');
===expect===
