===description===
Legacy native `resource` in a return type position must not be treated as an
undefined class while mir reports the underlying declaration error.
===config===
suppress=UnusedFunction
===file===
<?php
function makeHandle(): resource {
    return fopen('php://memory', 'r');
}
===expect===

