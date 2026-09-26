===cursor===
references
===file===
<?php
final class Counter {
    public int $count = 0;
    public function bump(): void { $this->count++; }
}
$c = new Counter();
echo $c->co<CURSOR>unt;
===expect===
test.php@4:42-4:47
test.php@7:9-7:14
