===description===
`SensitiveParameterValue` is generic over the value it wraps: the constructor
argument binds the type `getValue()` returns.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesString(string $value): void {}
function takesInt(int $value): void {}

$wrapped = new SensitiveParameterValue('secret');
takesString($wrapped->getValue());
takesInt($wrapped->getValue());
//       ^^^^^^^^^^^^^^^^^^^^ InvalidScalarArgument: Argument $value of takesInt() expects 'int', got scalar 'string'

$intWrapped = new SensitiveParameterValue(1);
takesInt($intWrapped->getValue());
