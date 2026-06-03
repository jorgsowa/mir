===description===
does not report function called via use function
===file:lib.php===
<?php
namespace Utils;

function helper(): void {}
===file:main.php===
<?php
use function Utils\helper;

helper();
===expect===
