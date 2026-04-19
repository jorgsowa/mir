===source===
<?php
function outer(): void {
    function inner(): void {
        nonexistent_function();
    }
}
===expect===
UndefinedFunction: Function nonexistent_function() is not defined
