===description===
Float-literal docblock types parse and accept a matching float return
===file===
<?php
/** @return 3.14 */
function pi() {
    return 3.14;
}

/** @return float */
function half() {
    return -0.5;
}

===expect===
