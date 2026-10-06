===description===
reports null returned from non nullable
===file===
<?php
function f(): string {
    return null;
//  ^^^^^^^^^^^^ InvalidReturnType: Return type 'null' is not compatible with declared 'string'
}
