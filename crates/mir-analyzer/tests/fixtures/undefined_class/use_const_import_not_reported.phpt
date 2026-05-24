===description===
use const import not reported
===file===
<?php
use const Vendor\Missing\SOME_CONST;
echo SOME_CONST;
===expect===
UndefinedConstant@3:6: Constant SOME_CONST is not defined
