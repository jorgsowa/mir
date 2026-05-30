===description===
trailing // @mir-ignore comment suppresses its own line
===file===
<?php
function test(): void {
    new NoSuchClass(); // @mir-ignore UndefinedClass
}
===expect===
