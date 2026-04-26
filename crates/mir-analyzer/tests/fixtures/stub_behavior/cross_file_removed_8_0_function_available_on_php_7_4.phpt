===config===
php_version=7.4
===file:TextHelper.php===
<?php
function format_hebrew(string $text): void {
    hebrevc($text);
}
===file:App.php===
<?php
format_hebrew('שלום');
===expect===
