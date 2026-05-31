===description===
class declared inside an if (! class_exists()) guard is indexed and usable
===file===
<?php
if (! class_exists('Money')) {
    class Money
    {
        public int $amount = 0;
    }
}

function make(): Money
{
    return new Money();
}
===expect===
