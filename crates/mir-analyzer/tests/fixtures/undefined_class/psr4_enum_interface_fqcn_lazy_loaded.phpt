===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/HasLabel.php===
<?php
namespace App;
interface HasLabel {
    public function label(): string;
}
===file:Status.php===
<?php
enum Status: string implements \App\HasLabel {
    case Active = 'active';
    public function label(): string { return $this->value; }
}
===expect===
