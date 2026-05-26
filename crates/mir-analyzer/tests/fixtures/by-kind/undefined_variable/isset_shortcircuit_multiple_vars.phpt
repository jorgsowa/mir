===description===
isset with multiple variables — all available in true branch
===file===
<?php
if (isset($x, $y, $z) && true) {
    echo $x . $y . $z; // all should be defined
}
===expect===
