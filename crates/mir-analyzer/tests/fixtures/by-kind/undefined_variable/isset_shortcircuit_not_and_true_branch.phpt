===description===
!isset short-circuit with && — variable undefined in true branch
===file===
<?php
if (!isset($x) && true) {
    echo $x;
}
===expect===
UndefinedVariable@3:9-3:11: Variable $x is not defined
