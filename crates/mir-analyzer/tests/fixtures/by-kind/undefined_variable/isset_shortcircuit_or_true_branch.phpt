===description===
isset short-circuit with || — in the true branch $x is set only when the left disjunct held
===file===
<?php
if (isset($x) || isset($y)) {
    echo $x;
//       ^^ PossiblyUndefinedVariable: Variable $x might not be defined
}
