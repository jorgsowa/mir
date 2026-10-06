===description===
Enum string or enum int incorrect int
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Ns;

/** @param ( "foo" | "bar" | 1 | 2 | 3 ) $s */
function foo($s) : void {}
foo(4);
//  ^ InvalidArgument: Argument $s of foo() expects '"foo"|"bar"|1|2|3', got '4'
