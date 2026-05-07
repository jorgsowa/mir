===description===
reports return null from void
===file===
<?php
function f(): void {
    return null;
}
===expect===
InvalidReturnType@3:4: Return type 'null' is not compatible with declared 'void'
