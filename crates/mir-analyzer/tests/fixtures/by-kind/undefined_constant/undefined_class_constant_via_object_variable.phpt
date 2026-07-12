===description===
$obj::MISSING (constant access through an object-instance variable) reports UndefinedConstant.
===file===
<?php
class A {}
function run(A $obj): void {
    echo $obj::MISSING;
}
===expect===
UndefinedConstant@4:9-4:22: Constant A::MISSING is not defined
