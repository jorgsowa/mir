===description===
A call to a method the winning declaration of a duplicated class lacks still counts
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file:a.php===
<?php
final class Location {
    public function line(): int {
        return $this->compute();
    }

    private function compute(): int { return 1; }
}
===file:b.php===
<?php
final class Location {
    public function line(): int { return 0; }
}
(new Location())->line();
