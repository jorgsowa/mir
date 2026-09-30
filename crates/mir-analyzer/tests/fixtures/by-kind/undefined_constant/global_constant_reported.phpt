===description===
global constant reported
===file===
<?php
function test(): void {
    echo UNDEFINED_CONST;
//       ^^^^^^^^^^^^^^^ UndefinedConstant: Constant UNDEFINED_CONST is not defined
}
===expect===
