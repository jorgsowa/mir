===description===
@mir-ignore with no kind name suppresses every issue on the target line
===file===
<?php
function test(): void {
    // @mir-ignore
    new NoSuchClass();
}
===expect===
