===description===
$obj::MISSING (constant access through an object-instance variable) reports UndefinedConstant.
===file===
<?php
class A {}
function run(A $obj): void {
    echo $obj::MISSING;
//       ^^^^^^^^^^^^^ UndefinedConstant: Constant A::MISSING is not defined
}
===expect===
