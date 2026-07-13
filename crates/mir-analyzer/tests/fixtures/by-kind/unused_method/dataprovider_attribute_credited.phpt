===description===
#[DataProvider] method is credited as used, not flagged
===config===
suppress=
===file===
<?php
#[Attribute]
class DataProvider {
    public function __construct(public string $methodName) {}
}

class BarTest {
    #[DataProvider('provideCases')]
    public function testAdd(int $a, int $b): void {
        echo $a + $b;
    }

    private static function provideCases(): array {
        return [[1, 2]];
    }
}
===expect===
