===config===
php_version=7.4
===file:TextHelper.php===
<?php
function test_wrong_type(int $n): void {
    hebrevc($n);
}
===file:App.php===
<?php
test_wrong_type(42);
===expect===
TextHelper.php: InvalidArgument: Argument $hebrew_text of hebrevc() expects 'string', got 'int'
