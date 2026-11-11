===description===
`numeric` accepts ints, floats and numeric string literals, and rejects other strings and bools.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/** @param numeric $value */
function acceptsNumeric($value): void {}

acceptsNumeric(1);
acceptsNumeric(1.5);
acceptsNumeric('123');
acceptsNumeric('1.5e3');
acceptsNumeric('abc');
//             ^^^^^ InvalidArgument: Argument $value of acceptsNumeric() expects 'numeric', got '"abc"'
acceptsNumeric(true);
//             ^^^^ InvalidArgument: Argument $value of acceptsNumeric() expects 'numeric', got 'true'
