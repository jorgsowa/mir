===description===
Get-only computed properties are initialized on access.
===config===
php_version=8.4
===file===
<?php
final class Bag
{
    public string $label;
    public function __construct(string $label)
      {
           $this->label = $label;
       }
}

final class View
{
    public readonly Bag $bag;
    public string $label { get => $this->bag->label; }
    public function __construct(Bag $bag)
      {
           $this->bag = $bag;
       }
}
===expect===
