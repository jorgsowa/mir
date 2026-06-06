===description===
Invalid private class const fetch
===file===
<?php
class A
{
    private const IS_PRIVATE = 1;
}

echo A::IS_PRIVATE;
===expect===
InaccessibleClassConstant@7:9-7:19: Cannot access constant A::IS_PRIVATE
