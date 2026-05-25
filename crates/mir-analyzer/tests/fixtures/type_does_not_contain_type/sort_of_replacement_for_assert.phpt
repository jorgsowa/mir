===description===
sortOfReplacementForAssert
===file===
<?php
namespace Bar;

/**
 * @param mixed $_b
 * @assert true $_b
 */
function myAssert($_b) : void {
    if ($_b !== true) {
        throw new Exception("bad");
    }
}

function bar(?string $s) : string {
    myAssert($s);
    return $s;
}
===expect===
TypeDoesNotContainType
===ignore===
TODO
