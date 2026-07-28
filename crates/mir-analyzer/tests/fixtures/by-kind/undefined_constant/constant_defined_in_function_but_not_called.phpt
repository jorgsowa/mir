===description===
A define() inside a function body is indexed unconditionally, regardless of
whether the function is ever called — mir does not do call-graph reachability
analysis, matching the existing guarded-function-declaration precedent.
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
