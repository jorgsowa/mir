===description===
In non-strict PHP, passing int/float to a string-typed parameter is a benign coercion.
Should emit ArgumentTypeCoercion (Info), not InvalidArgument (Error).
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param string $s */
function takes_string(string $s): void { echo $s; }

takes_string(1);
//           ^ ArgumentTypeCoercion: Argument $s of takes_string() expects 'string', got '1' — coercion may fail at runtime
takes_string(42);
//           ^^ ArgumentTypeCoercion: Argument $s of takes_string() expects 'string', got '42' — coercion may fail at runtime
takes_string(3.14);
//           ^^^^ ArgumentTypeCoercion: Argument $s of takes_string() expects 'string', got '3.14' — coercion may fail at runtime
