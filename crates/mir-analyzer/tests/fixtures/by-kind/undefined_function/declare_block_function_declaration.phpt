===description===
function declared inside a declare() {} block is indexed and callable
===file===
<?php
declare(ticks=1) {
    function tick_helper(): string
    {
        return 'tick';
    }
}

function wrap(): string
{
    return tick_helper();
}
===expect===
