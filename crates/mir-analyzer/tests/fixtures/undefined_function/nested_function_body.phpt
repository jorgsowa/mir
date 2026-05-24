===description===
nested function body
===file===
<?php
function outer(): void {
    function inner(): void {
        nonexistent_function();
    }
}
===expect===
UndefinedFunction@4:9: Function nonexistent_function() is not defined
