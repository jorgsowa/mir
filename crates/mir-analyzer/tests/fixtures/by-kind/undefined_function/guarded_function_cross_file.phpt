===description===
guarded helper declared in one file resolves when called from another (Laravel helpers.php pattern)
===file:helpers.php===
<?php
if (! function_exists('my_helper')) {
    function my_helper(): string
    {
        return 'hi';
    }
}
===file:main.php===
<?php
function wrap(): string
{
    return my_helper();
}
===expect===
