===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Formatter.php===
<?php
namespace App;

class Formatter {
    public function format(\DateTimeImmutable $dt): string {
        return $dt->format('Y-m-d H:i:s');
    }

    public function keys(array $data): array {
        // $filter_value is optional — one-arg call must be accepted
        return array_keys($data);
    }
}
===file:Main.php===
<?php
// Extend triggers PSR-4 lazy-load of App\Formatter, which uses stdlib types
class LogFormatter extends \App\Formatter {}

$f = new LogFormatter();
$result = $f->format(new \DateTimeImmutable());
$keys = $f->keys(['a' => 1, 'b' => 2]);
===expect===
