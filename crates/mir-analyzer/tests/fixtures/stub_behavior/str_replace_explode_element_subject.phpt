===description===
str_replace with array-element subject (explode()[n]) returns string, not string|array
===file===
<?php
function colorAlpha(string $colorString): float {
    return (float) str_replace(')', '', explode(',', $colorString)[3]);
}
===expect===
