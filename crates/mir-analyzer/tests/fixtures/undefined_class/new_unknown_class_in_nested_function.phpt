===source===
<?php
function outer(): void {
    function inner(): void {
        new UnknownClass();
    }
}
===expect===
UndefinedClass: UnknownClass
