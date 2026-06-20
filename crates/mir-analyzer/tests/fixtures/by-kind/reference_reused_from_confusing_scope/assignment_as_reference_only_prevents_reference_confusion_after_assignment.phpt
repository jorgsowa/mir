===description===
Assignment as reference only prevents reference confusion after assignment
===config===
suppress=MixedAssignment,UnusedForeachValue
===file===
<?php
$arr = [1, 2, 3];
foreach ($arr as &$i) {
    ++$i;
}

for ($i = 0; $i < 10; ++$i) {
    echo $i;
}

$i = &$foo;

===expect===
UndefinedVariable@11:6-11:10: Variable $foo is not defined
