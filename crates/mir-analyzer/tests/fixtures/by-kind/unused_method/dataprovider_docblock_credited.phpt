===description===
@dataProvider method is credited as used, not flagged
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class FooTest {
//<^ +11:1 UnusedClass: Class FooTest is never referenced
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
