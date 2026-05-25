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
InaccessibleClassConstant
===ignore===
TODO
