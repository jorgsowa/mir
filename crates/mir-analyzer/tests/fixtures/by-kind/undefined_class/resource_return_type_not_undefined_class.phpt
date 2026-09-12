===description===
Legacy `resource` return types are not undefined classes.
===config===
suppress=UnusedFunction
===file===
<?php
function makeHandle(): resource {
    return fopen('php://memory', 'r');
}
===expect===
