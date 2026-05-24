===description===
isset short-circuit with || — no narrowing in true branch for unset variable
===file===
<?php
if (isset($x) || isset($y)) {
    echo $x;
}
===expect===
UndefinedVariable@3:10: Variable $x is not defined
