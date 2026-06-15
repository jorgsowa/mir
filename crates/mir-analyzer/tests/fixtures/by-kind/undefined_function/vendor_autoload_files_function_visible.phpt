===description===
A function defined in a vendor autoload.files entry is visible without explicit indexing.
Composer's autoload.files lists files that define global functions and constants;
AnalysisSession lazy-loads them automatically on the first analysis call.
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:vendor/composer/autoload_files.php===
<?php
$vendorDir = dirname(__DIR__);
return array(
    'abc123' => $vendorDir . '/helpers/functions.php',
);
===file:vendor/helpers/functions.php===
<?php
function vendor_str_pad(string $s, int $len): string {
    return str_pad($s, $len);
}
===file:src/Consumer.php===
<?php
namespace App;
class Consumer {
    public function run(): string {
        return vendor_str_pad('hello', 10);
    }
}
===expect===
