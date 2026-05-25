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
UndefinedConstant@9:6: Constant CONSTANT is not defined
