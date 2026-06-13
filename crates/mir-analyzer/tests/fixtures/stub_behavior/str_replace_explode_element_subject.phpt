===description===
str_replace with array-element subject (explode()[n]) returns string, not string|array
===file===
<?php
function colorAlpha(string $colorString): float {
    return (float) str_replace(')', '', explode(',', $colorString)[3]);
}
===expect===
PossiblyInvalidArrayAccess@3:41-3:70: Possibly invalid array access: 'array<int, string>|false' might not support []
