===source===
<?php
function outer(): void {
    function inner(): void {
        nonexistent_function();
    }
}
===expect===
UndefinedFunction: nonexistent_function()
