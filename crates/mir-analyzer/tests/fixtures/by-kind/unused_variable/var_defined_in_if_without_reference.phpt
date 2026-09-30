===description===
Var defined in if without reference
===file===
<?php
$a = 5;
//<^^ UnusedVariable: Variable $a is never read
if (rand(0, 1)) {
    $b = "hello";
//  ^^ UnusedVariable: Variable $b is never read
} else {
    $b = "goodbye";
}
===expect===
