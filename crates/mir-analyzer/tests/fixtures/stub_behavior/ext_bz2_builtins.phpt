===description===
`bzcompress`/`bzdecompress` are ext-bz2 built-in functions (a required extension); stubs are missing.
===config===
php_version=8.4
===file===
<?php
function pack_(string $data): void {
    $c = bzcompress($data);
    $d = bzdecompress('test');
    echo $c;
    echo $d;
}
===expect===
