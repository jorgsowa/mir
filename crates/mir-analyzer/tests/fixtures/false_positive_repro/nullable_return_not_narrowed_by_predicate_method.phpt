===description===
Valid PHP: after `if (!$this->isAssigned()) { throw ...; }` the property
`$this->value` is provably non-null. mir cannot narrow the property through the
boolean-predicate method call, keeps `int|null`, and reports the `int` return as
nullable.
===ignore===
===config===
php_version=8.4
===file===
<?php
final class Box
{
    private ?int $value = null;
    public function assign(int $value): void
      {
           $this->value = $value;
       }

    public function isAssigned(): bool
      {
        return $this->value !== null;
      }

       /** @return int */
    public function get(): int
      {
        if (!$this->isAssigned()) {
             throw new RuntimeException('value not assigned');
           }

        return $this->value;
      }
}
===expect===
