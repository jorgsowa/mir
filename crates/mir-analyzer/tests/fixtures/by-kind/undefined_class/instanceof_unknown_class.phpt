===description===
instanceof unknown class
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test($x): bool {
    return $x instanceof NoSuchClass;
//                       ^^^^^^^^^^^ UndefinedClass: Class NoSuchClass does not exist
}
