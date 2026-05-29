===description===
global constant reported
===file===
<?php
function test(): void {
    echo UNDEFINED_CONST;
}
===expect===
UndefinedConstant@3:10-3:25: Constant UNDEFINED_CONST is not defined
