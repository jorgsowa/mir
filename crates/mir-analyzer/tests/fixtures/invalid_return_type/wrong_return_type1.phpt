===description===
Wrong return type1
===file===
<?php
function fooFoo(): string {
    return 5;
}
===expect===
InvalidReturnStatement
===ignore===
TODO
