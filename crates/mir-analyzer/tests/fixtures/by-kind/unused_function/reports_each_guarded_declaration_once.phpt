===description===
A function declared under `function_exists` guards in two files is reported once per declaration
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file:a.php===
<?php
if (!function_exists('polyfilled')) {
    function polyfilled(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function polyfilled() is never called
}
===file:b.php===
<?php
if (!function_exists('polyfilled')) {
    function polyfilled(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function polyfilled() is never called
}
