===config===
php_version=8.0
===file:ArrayHelper.php===
<?php
function check_is_list(array $items): void {
    array_is_list($items);
}
===file:App.php===
<?php
check_is_list([1, 2, 3]);
===expect===
ArrayHelper.php: UndefinedFunction: Function array_is_list() is not defined
