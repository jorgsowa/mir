===description===
Catch does not return
===file===
<?php
function missing_return() : bool {
//                          ^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'bool'
    try {
    } finally {
    }
}
===expect===
