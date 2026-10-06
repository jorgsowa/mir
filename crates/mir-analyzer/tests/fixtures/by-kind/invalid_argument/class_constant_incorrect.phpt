===description===
Class constant incorrect
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Ns;

class C {
    const A = "bat";
    const B = "baz";
}
/** @param "foo"|"bar"|C::A|C::B $s */
function foo($s) : void {}
foo("for");
//  ^^^^^ InvalidArgument: Argument $s of foo() expects '"foo"|"bar"|"bat"|"baz"', got '"for"'
