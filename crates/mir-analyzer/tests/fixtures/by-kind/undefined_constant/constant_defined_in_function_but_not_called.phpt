===description===
Constant defined in function but not called
===file===
<?php
/**
 * @return void
 */
function defineConstant() {
    define("CONSTANT", 1);
}

echo CONSTANT;
===expect===
UndefinedConstant@9:5-9:13: Constant CONSTANT is not defined
