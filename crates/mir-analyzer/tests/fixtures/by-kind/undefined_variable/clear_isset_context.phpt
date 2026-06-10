===description===
Clear isset context
===file===
<?php
function greet(bool $arg): ?string
{
    return $arg ? "hi" : null;
}

echo greet($undef) ?? "bye";
===expect===
