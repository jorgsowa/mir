===description===
print() with a constant string is not flagged.
===file===
<?php
function test(): void {
    print("hello");
}
===expect===
