===source===
<?php
function test(): void {
    echo UNDEFINED_CONST;
}
===expect===
UndefinedConstant: Constant UNDEFINED_CONST is not defined
