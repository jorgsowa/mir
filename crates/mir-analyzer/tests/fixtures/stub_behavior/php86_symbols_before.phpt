===description===
The symbols PHP 8.6 adds don't exist for an 8.5 target.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function describe(ReflectionAttribute $attr): string {
    new Time\Duration();
//      ^^^^^^^^^^^^^ UndefinedClass: Class Time\Duration does not exist
    $p = gmp_prevprime(10);
//       ^^^^^^^^^^^^^^^^^ UndefinedFunction: Function gmp_prevprime() is not defined
    return $attr->getShortName() . gmp_strval($p);
//         ^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method ReflectionAttribute::getShortName() does not exist
}
