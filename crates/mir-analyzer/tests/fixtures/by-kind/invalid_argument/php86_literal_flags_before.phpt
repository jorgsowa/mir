===description===
Before PHP 8.6 an unknown flag for array_filter(), array_change_key_case(), pathinfo() or scandir() isn't reported.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
function flags(array $a, string $path): void {
    array_filter($a, null, 3);
    array_change_key_case($a, 2);
    echo pathinfo($path, PATHINFO_DIRNAME | PATHINFO_BASENAME);
    scandir($path, 3);
}
