===description===
does not report function called via use function
===config===
find_dead_code=true
===file:lib.php===
<?php
namespace Utils;

function helper(): void {}
===file:main.php===
<?php
use function Utils\helper;

helper();
===expect===
===ignore===
TODO
