===description===
@dataProvider method is credited as used, not flagged
===config===
suppress=
===file===
<?php
class FooTest {
    /**
     * @dataProvider provideCases
     */
    public function testAdd(int $a, int $b): void {
        echo $a + $b;
    }

    private static function provideCases(): array {
        return [[1, 2]];
    }
}
===expect===
