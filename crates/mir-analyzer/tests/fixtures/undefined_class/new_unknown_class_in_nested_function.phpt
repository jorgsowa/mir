===description===
new unknown class in nested function
===file===
<?php
function outer(): void {
    function inner(): void {
        new UnknownClass();
    }
}
===expect===
UndefinedClass: Class UnknownClass does not exist
===ignore===
TODO
