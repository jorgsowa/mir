===description===
Invalid protected class const fetch
===file===
<?php
class A
{
    protected const IS_PROTECTED = 1;
}

echo A::IS_PROTECTED;
===expect===
InaccessibleClassConstant@7:8-7:20: Cannot access constant A::IS_PROTECTED
