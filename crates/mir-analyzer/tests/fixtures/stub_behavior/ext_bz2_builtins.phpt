===description===
`bzcompress`/`bzdecompress` are ext-bz2 built-in functions (a required extension); stubs are missing.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
function pack_(string $data): void {
    $c = bzcompress($data);
    $d = bzdecompress('test');
    echo $c;
    echo $d;
}
===expect===
