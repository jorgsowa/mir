===description===
constantDefinedInFunctionButNotCalled
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
UndefinedConstant@9:5: Constant CONSTANT is not defined
