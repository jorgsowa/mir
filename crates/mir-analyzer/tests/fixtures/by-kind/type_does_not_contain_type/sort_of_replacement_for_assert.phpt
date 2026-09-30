===description===
Sort of replacement for assert
===config===
suppress=MissingThrowsDocblock
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
//                ^^^^^^^^^ UndefinedClass: Class Bar\Exception does not exist
    }
}

function bar(?string $s) : string {
    myAssert($s);
    return $s;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'true' is not compatible with declared 'string'
}
===expect===
