===description===
Clear isset context
===ignore===
TODO
===file===
<?php
function greet(bool $arg): ?string
{
    return $arg ? "hi" : null;
}

echo greet($undef) ?? "bye";
===expect===
