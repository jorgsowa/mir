===description===
guarded function keeps its declared return type (not mixed) so chained calls type-check
===file===
<?php
if (! function_exists('greeting')) {
    function greeting(): string
    {
        return 'hi';
    }
}

function wrap(): void
{
    $g = greeting();
    /** @mir-check $g is string */
    echo $g;
}
===expect===
