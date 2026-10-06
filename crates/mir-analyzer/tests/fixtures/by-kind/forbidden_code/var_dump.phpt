===description===
ForbiddenCode fires when calling a configured forbidden function, case-insensitively.
===config===
<mir>
  <forbiddenFunctions>
    <function name="var_dump"/>
  </forbiddenFunctions>
</mir>
===file===
<?php
function debug(mixed $v): void {
    var_dump($v);
//  ^^^^^^^^^^^^ ForbiddenCode: Use of var_dump is forbidden
    VAR_DUMP($v);
//  ^^^^^^^^^^^^ ForbiddenCode: Use of VAR_DUMP is forbidden
//  ^^^^^^^^ WrongCaseFunction: Function name 'VAR_DUMP' has incorrect casing; use 'var_dump'
    \var_dump($v);
//  ^^^^^^^^^^^^^ ForbiddenCode: Use of var_dump is forbidden
}
