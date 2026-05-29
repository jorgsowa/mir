===description===
cross file substr count too few params 5 3
===config===
php_version=5.3
===file:StringHelper.php===
<?php
function countWord(string $output): int {
    // In PHP 5.3, substr_count requires offset parameter (3 required args total)
    return substr_count($output, 'info');
}
===file:App.php===
<?php
$result = countWord('some text with info here');
===expect===
StringHelper.php: TooFewArguments@4:12-4:41: Too few arguments for substr_count(): expected 3, got 2
