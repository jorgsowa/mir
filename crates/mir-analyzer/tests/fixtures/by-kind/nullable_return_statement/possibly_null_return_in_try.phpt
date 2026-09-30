===description===
Possibly null return in try
===config===
suppress=MissingThrowsDocblock,UnusedVariable
===file===
<?php
function foo() : string {
    $a = null;

    try {
        $a = dangerous();
    } catch (Exception $e) {
        return $a;
//      ^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
    }

    return $a;
}

function dangerous() : string {
    if (rand(0, 1)) {
        throw new Exception("bad");
    }
    return "hello";
}
===expect===
