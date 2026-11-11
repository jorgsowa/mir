===description===
The literals "" and "0" are not non-falsy-string; "0" is still a non-empty-string.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-falsy-string $value */
function takesNonFalsy(string $value): void {}
/** @param non-empty-string $value */
function takesNonEmpty(string $value): void {}

takesNonFalsy('');
//            ^^ InvalidArgument: Argument $value of takesNonFalsy() expects 'non-falsy-string', got '""'
takesNonFalsy('0');
//            ^^^ InvalidArgument: Argument $value of takesNonFalsy() expects 'non-falsy-string', got '"0"'
takesNonFalsy('00');
takesNonFalsy('a');
takesNonEmpty('0');
