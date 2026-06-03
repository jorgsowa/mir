===description===
Theres no catch
===file===
<?php
function missing_return() : bool {
    try {
    } finally {
    }
}
===expect===
InvalidReturnType@2:34-6:35: Return type 'void' is not compatible with declared 'bool'
