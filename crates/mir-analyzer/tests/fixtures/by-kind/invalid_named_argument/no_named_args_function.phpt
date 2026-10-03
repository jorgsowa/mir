===description===
No named args function
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @no-named-arguments */
function takesArguments(string $name, int $age) : void {}

takesArguments(age: 5, name: "hello");
//             ^^^^^^ InvalidNamedArguments: takesArguments() does not accept named arguments
//                     ^^^^^^^^^^^^^ InvalidNamedArguments: takesArguments() does not accept named arguments
===expect===
