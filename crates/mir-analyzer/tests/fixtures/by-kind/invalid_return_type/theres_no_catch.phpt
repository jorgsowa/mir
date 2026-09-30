===description===
Theres no catch
===file===
<?php
function missing_return() : bool {
//                               ^ +4:1 InvalidReturnType: Return type 'void' is not compatible with declared 'bool'
    try {
    } finally {
    }
}
===expect===
