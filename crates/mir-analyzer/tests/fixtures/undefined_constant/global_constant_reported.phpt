===description===
global constant reported
===file===
<?php
function test(): void {
    echo UNDEFINED_CONST;
}
===expect===
UndefinedConstant@3:9: Constant UNDEFINED_CONST is not defined
