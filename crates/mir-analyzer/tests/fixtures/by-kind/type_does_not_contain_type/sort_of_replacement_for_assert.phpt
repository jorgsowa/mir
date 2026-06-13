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
    }
}

function bar(?string $s) : string {
    myAssert($s);
    return $s;
}
===expect===
UndefinedClass@10:19-10:28: Class Bar\Exception does not exist
InvalidReturnType@16:5-16:15: Return type 'true' is not compatible with declared 'string'
