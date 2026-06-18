===description===
String encoding/transformation functions preserve non-empty-string when the input is non-empty.
htmlspecialchars, urlencode, base64_encode, nl2br, str_rot13, etc.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-string $s */
function test_htmlspecialchars(string $s): void {
    $r = htmlspecialchars($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_htmlentities(string $s): void {
    $r = htmlentities($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_urlencode(string $s): void {
    $r = urlencode($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_rawurlencode(string $s): void {
    $r = rawurlencode($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_base64_encode(string $s): void {
    $r = base64_encode($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_nl2br(string $s): void {
    $r = nl2br($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_addslashes(string $s): void {
    $r = addslashes($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_str_rot13(string $s): void {
    $r = str_rot13($s);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_plain_string(?string $s): void {
    $r = urlencode((string) $s);
    /** @mir-check $r is string */
    $_ = $r;
}
===expect===
