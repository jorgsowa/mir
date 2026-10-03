===description===
Invalid private class const fetch from subclass
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A
{
    private const IS_PRIVATE = 1;
}

class B extends A
{
    function fooFoo(): int {
        return A::IS_PRIVATE;
//                ^^^^^^^^^^ InaccessibleClassConstant: Cannot access constant A::IS_PRIVATE
    }
}
===expect===
