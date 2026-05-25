===description===
use const import not reported
===config===
suppress=UndefinedConstant
===file===
<?php
use const Vendor\Missing\SOME_CONST;
echo SOME_CONST;
===expect===
