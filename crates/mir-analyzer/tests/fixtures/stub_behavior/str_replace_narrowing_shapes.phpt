===description===
str_replace narrowing for various string-subject shapes
===file===
<?php

// Pattern 1: explode result element directly
function test1(string $s): float {
    return (float) str_replace(')', '', explode(',', $s)[3]);
}

// Pattern 2: via intermediate variable
function test2(string $s): float {
    $parts = explode(',', $s);
    return (float) str_replace(')', '', $parts[3]);
}

// Pattern 4: element of array param typed as string[]
/** @param string[] $parts */
function test4(array $parts): float {
    return (float) str_replace(')', '', $parts[0]);
}

// Pattern 5: str_ireplace with explode element
function test5(string $colorString): float {
    return (float) str_ireplace('rgb(', '', explode(',', $colorString)[0]);
}
===expect===
PossiblyInvalidArrayAccess@5:41-5:60: Possibly invalid array access: 'array<int, string>|false' might not support []
PossiblyInvalidArrayAccess@11:41-11:50: Possibly invalid array access: 'array<int, string>|false' might not support []
PossiblyInvalidArrayAccess@22:45-22:74: Possibly invalid array access: 'array<int, string>|false' might not support []
