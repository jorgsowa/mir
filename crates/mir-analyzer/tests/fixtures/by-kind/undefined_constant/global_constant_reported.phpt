===description===
global constant reported
===file===
<?php
function test(): void {
    echo UNDEFINED_CONST;
}
===expect===
UndefinedConstant@3:9-3:24: Constant UNDEFINED_CONST is not defined
