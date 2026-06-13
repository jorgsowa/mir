===description===
Verify explode()[n] element type and str_replace return with that subject
===file===
<?php
function colorAlpha(string $colorString): float {
    $parts = explode(',', $colorString);
    $elem = $parts[3];
    /** @mir-check $elem is string */
    $replaced = str_replace(')', '', $elem);
    /** @mir-check $replaced is string */
    return (float) $replaced;
}

function directAccess(string $colorString): float {
    $elem = explode(',', $colorString)[3];
    /** @mir-check $elem is string */
    $replaced = str_replace(')', '', $elem);
    /** @mir-check $replaced is string */
    return (float) $replaced;
}
===expect===
PossiblyInvalidArrayAccess@4:13-4:22: Possibly invalid array access: 'array<int, string>|false' might not support []
PossiblyInvalidArrayAccess@12:13-12:42: Possibly invalid array access: 'array<int, string>|false' might not support []
