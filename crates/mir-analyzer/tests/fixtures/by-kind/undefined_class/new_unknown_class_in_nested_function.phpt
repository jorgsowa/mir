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
UndefinedClass@4:13: Class UnknownClass does not exist
