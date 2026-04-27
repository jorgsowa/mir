===file:HasLabel.php===
<?php
namespace App;
interface HasLabel {
    public function label(): string;
}
===file:Status.php===
<?php
use App\HasLabel;
enum Status: string implements HasLabel {
    case Active = 'active';
    case Inactive = 'inactive';
    public function label(): string { return $this->value; }
}
===expect===
