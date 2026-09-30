===description===
!isset short-circuit with && — variable undefined in true branch
===file===
<?php
if (!isset($x) && true) {
    echo $x;
//       ^^ UndefinedVariable: Variable $x is not defined
}
===expect===
