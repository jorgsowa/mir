===description===
isset short-circuit with && — variable available in true branch
===file===
<?php
if (isset($x) && true) {
    echo $x;
}
===expect===
