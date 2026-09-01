===description===
Valid PHP 8.4: a get-only property whose `get =>` hook returns another property
is never uninitialized — the value is computed on access and never stored, so mir
does not report it as possibly uninitialized.
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
