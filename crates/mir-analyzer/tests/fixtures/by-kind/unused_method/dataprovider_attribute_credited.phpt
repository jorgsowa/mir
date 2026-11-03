===description===
#[DataProvider] method is credited as used, not flagged
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
#[Attribute]
class DataProvider {
    public function __construct(public string $methodName) {}
}

class BarTest {
//<^ +9:1 UnusedClass: Class BarTest is never referenced
    #[DataProvider('provideCases')]
    public function testAdd(int $a, int $b): void {
        echo $a + $b;
    }

    private static function provideCases(): array {
        return [[1, 2]];
    }
}
