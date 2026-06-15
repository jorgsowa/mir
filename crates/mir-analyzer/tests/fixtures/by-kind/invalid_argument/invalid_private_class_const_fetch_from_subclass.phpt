===description===
Invalid private class const fetch from subclass
===config===
suppress=MixedReturnStatement
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
    }
}
===expect===
InaccessibleClassConstant@10:18-10:28: Cannot access constant A::IS_PRIVATE
