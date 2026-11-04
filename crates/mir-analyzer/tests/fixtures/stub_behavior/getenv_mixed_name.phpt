===description===
getenv() with a mixed name returns string|false
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(mixed $name): string|false {
    $value = getenv($name);
    /** @mir-check $value is string|false */
    return $value;
}
