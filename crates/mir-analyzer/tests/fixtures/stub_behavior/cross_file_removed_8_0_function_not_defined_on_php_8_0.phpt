===description===
cross file removed 8 0 function not defined on php 8 0
===config===
php_version=8.0
===file:TextHelper.php===
<?php
function format_hebrew(string $text): void {
    hebrevc($text);
}
===file:App.php===
<?php
format_hebrew('שלום');
===expect===
TextHelper.php: UndefinedFunction@3:5: Function hebrevc() is not defined
