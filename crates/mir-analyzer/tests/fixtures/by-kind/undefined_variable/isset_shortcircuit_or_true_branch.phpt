===description===
isset short-circuit with || — no narrowing in true branch for unset variable
===file===
<?php
if (isset($x) || isset($y)) {
    echo $x;
//       ^^ UndefinedVariable: Variable $x is not defined
}
