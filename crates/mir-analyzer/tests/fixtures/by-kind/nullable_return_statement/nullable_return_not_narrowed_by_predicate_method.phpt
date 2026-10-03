===description===
Predicate-method guards narrow properties before return.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
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
