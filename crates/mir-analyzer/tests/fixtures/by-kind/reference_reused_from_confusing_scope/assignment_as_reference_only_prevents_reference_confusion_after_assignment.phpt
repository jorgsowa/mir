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
UnsupportedReferenceUsage@11:1-11:11: Reference assignment is not supported
UndefinedVariable@11:7-11:11: Variable $foo is not defined
