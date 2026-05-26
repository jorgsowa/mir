===description===
is_float, is_double, is_real all narrow to float type
===file===
<?php
function testIsFloat($x) {
    if (is_float($x)) {
        return $x + 1.5;
    }
}

function testIsDouble($x) {
    if (is_double($x)) {
        return $x * 2.0;
    }
}

function testIsReal($x) {
    if (is_real($x)) {
        return sqrt($x);
    }
}
===expect===
