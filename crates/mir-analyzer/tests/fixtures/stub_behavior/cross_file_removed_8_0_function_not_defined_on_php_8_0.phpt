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
TextHelper.php: UndefinedFunction: Function hebrevc() is not defined
