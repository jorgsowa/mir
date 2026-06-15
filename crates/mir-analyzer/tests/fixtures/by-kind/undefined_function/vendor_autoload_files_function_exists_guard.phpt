===description===
A vendor autoload.files function defined inside an if(!function_exists(...)) guard
(the common Laravel pattern) is still visible after lazy loading.
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:vendor/composer/autoload_files.php===
<?php
$vendorDir = dirname(__DIR__);
return array(
    'def456' => $vendorDir . '/helpers/functions.php',
);
===file:vendor/helpers/functions.php===
<?php
if (! function_exists('guarded_helper')) {
    function guarded_helper(string $s): string {
        return strtolower($s);
    }
}
===file:src/Consumer.php===
<?php
namespace App;
class Consumer {
    public function label(): string {
        return guarded_helper('Hello');
    }
}
===expect===
