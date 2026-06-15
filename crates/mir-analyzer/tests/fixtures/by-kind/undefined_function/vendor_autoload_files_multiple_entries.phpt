===description===
Functions from multiple vendor autoload.files entries are all lazy-loaded and visible.
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:vendor/composer/autoload_files.php===
<?php
$vendorDir = dirname(__DIR__);
return array(
    'aaa111' => $vendorDir . '/pkg-a/helpers.php',
    'bbb222' => $vendorDir . '/pkg-b/helpers.php',
);
===file:vendor/pkg-a/helpers.php===
<?php
function pkg_a_helper(int $n): int {
    return $n * 2;
}
===file:vendor/pkg-b/helpers.php===
<?php
function pkg_b_helper(string $s): bool {
    return strlen($s) > 0;
}
===file:src/Consumer.php===
<?php
namespace App;
class Consumer {
    public function run(): bool {
        $n = pkg_a_helper(5);
        return pkg_b_helper((string) $n);
    }
}
===expect===
