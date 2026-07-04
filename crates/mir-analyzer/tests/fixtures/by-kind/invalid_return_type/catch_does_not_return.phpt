===description===
Catch does not return
===file===
<?php
function missing_return() : bool {
    try {
    } finally {
    }
}
===expect===
InvalidReturnType@2:33-6:1: Return type 'void' is not compatible with declared 'bool'
