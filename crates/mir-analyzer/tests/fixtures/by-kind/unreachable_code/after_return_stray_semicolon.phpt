===description===
A stray semicolon after return is an empty statement and is unreachable.
===file===
<?php
function f(): void {
    return;;
//         ^ UnreachableCode: Unreachable code detected
}
