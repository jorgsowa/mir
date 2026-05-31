===description===
function declared inside an if (! function_exists()) guard is indexed and callable
===file===
<?php
if (! function_exists('my_helper')) {
    function my_helper(): string
    {
        return 'hi';
    }
}

function wrap(): string
{
    return my_helper();
}
===expect===
