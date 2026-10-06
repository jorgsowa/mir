===description===
Bad by ref
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fooFoo(string &$v): void {}
fooFoo("a");
//     ^^^ InvalidPassByReference: Argument $v of fooFoo() must be passed by reference
