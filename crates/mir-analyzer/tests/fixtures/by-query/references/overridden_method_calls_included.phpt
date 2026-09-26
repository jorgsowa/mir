===description===
Calls to an override count as references to the overridden method, so a rename covers the hierarchy.
===cursor===
references
===file===
<?php
class Base {
    public function greet(): string { return 'hi'; }
}
final class Child extends Base {
    public function greet(): string { return 'hey'; }
}
(new Base())->gr<CURSOR>eet();
(new Child())->greet();
===expect===
test.php@8:14-8:19
test.php@9:15-9:20
