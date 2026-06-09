===description===
String as a boundary
===file===
<?php
/**
 * @param int<0, "bar"> $a
 */
function scope(int $a){
    return $a;
}
===expect===
