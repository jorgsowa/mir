===description===
Builtin functioninvalid argument with weak types
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$s = substr(5, 4);
//          ^ ArgumentTypeCoercion: Argument $string of substr() expects 'string', got '5' — coercion may fail at runtime
