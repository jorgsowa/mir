===description===
`?int` property is non-null inside an `if ($this->id !== null)` guard.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
class Box {
    private ?int $id = null;
    public function get(): int {
        if ($this->id !== null) {
            return $this->id;
        }
        return 0;
    }
}
===expect===
