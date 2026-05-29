===description===
!isset short-circuit with && — variable undefined in true branch
===file===
<?php
if (!isset($x) && true) {
    echo $x;
}
===expect===
UndefinedVariable@3:10-3:12: Variable $x is not defined
