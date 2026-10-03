===description===
Wrong type variadic arguments
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesArguments(int ...$args) : void {}

takesArguments(age: "abc");
//             ^^^^^^^^^^ InvalidArgument: Argument $args of takesArguments() expects 'int', got '"abc"'
===expect===
