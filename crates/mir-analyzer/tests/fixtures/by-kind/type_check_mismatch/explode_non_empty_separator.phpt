===description===
explode with a non-empty separator always returns non-empty-list<string>.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_literal_separator(): void {
    $parts = explode('/', '/foo/bar');
    /** @mir-check $parts is non-empty-list<string> */
    $_ = $parts;
}

function test_space_separator(): void {
    $parts = explode(' ', 'hello world');
    /** @mir-check $parts is non-empty-list<string> */
    $_ = $parts;
}
===expect===
