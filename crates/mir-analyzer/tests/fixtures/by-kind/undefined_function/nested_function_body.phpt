===description===
nested function body
===file===
<?php
function outer(): void {
    function inner(): void {
        nonexistent_function();
//      ^^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function nonexistent_function() is not defined
    }
}
