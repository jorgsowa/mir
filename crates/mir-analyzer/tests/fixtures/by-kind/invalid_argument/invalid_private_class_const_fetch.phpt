===description===
Invalid private class const fetch
===file===
<?php
class A
{
    private const IS_PRIVATE = 1;
}

echo A::IS_PRIVATE;
//      ^^^^^^^^^^ InaccessibleClassConstant: Cannot access constant A::IS_PRIVATE
