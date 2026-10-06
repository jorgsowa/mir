===description===
Missing param type shouldnt prevent undefined class error
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @suppress MissingParamType */
function foo($s = Foo::BAR) : void {}
//                ^^^ UndefinedClass: Class Foo does not exist
